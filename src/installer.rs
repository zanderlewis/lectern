use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use fs_extra::dir::{CopyOptions, copy as copy_dir};
use futures::stream::{FuturesUnordered, StreamExt};
use git2::Repository;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tempfile::NamedTempFile;
use tokio::fs;
use tokio::sync::Semaphore;
use tokio::task;
use zip::ZipArchive;
use sha2::Digest;

use crate::model::LockedPackage;
use crate::utils;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct InstalledPackage {
    pub name: String,
    pub version: String,
    pub path: Utf8PathBuf,
}

const NETWORK_FACTOR: usize = 8;
const CPU_FACTOR: usize = 4;

fn get_package_cache_dir() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".lectern_cache")
        .join("packages")
}

fn get_cached_package_path(name: &str, version: &str, url: &str) -> PathBuf {
    let mut hasher = sha2::Sha256::new();
    hasher.update(format!("{}-{}-{}", name, version, url).as_bytes());
    let hash = format!("{:x}", hasher.finalize());
    
    get_package_cache_dir().join(format!("{}.zip", hash))
}

pub async fn install_packages(
    pkgs: &[LockedPackage],
    project_dir: &Path,
) -> Result<Vec<InstalledPackage>> {
    let vendor = project_dir.join("vendor");
    fs::create_dir_all(&vendor).await?;

    let cores = num_cpus::get();
    let net_sem = Arc::new(Semaphore::new(cores * NETWORK_FACTOR));
    let cpu_sem = Arc::new(Semaphore::new(cores * CPU_FACTOR));

    let client = reqwest::Client::builder()
        .user_agent("lectern/0.1")
        .build()?;

    // spawn tasks and gather
    let mut futs = FuturesUnordered::new();
    #[allow(clippy::unnecessary_to_owned)]
    for p in pkgs.to_owned() {
        let vendor = vendor.clone();
        let client = client.clone();
        let net = net_sem.clone();
        let cpu = cpu_sem.clone();
        futs.push(tokio::spawn(async move {
            install_single(&p, &vendor, &client, net, cpu).await
        }));
    }

    let mut out = Vec::new();
    while let Some(res) = futs.next().await {
        let r = res.context("task join")??;
        out.push(r);
    }
    Ok(out)
}

async fn install_single(
    p: &LockedPackage,
    vendor: &Path,
    client: &reqwest::Client,
    net_sem: Arc<Semaphore>,
    cpu_sem: Arc<Semaphore>,
) -> Result<InstalledPackage> {
    let target = vendor.join(
        p.name
            .replace('/', std::path::MAIN_SEPARATOR.to_string().as_str()),
    );
    if target.exists() {
        return Ok(InstalledPackage {
            name: p.name.clone(),
            version: p.version.clone(),
            path: Utf8PathBuf::from_path_buf(target).unwrap(),
        });
    }

    utils::print_info(&format!("Installing {}: {}", p.name, p.version));

    fs::create_dir_all(&target).await?;

    // Prefer dist (zip) over source (git) for better performance
    if let Some(dist_info) = &p.dist {
        // Handle distribution packages with caching
        download_and_unpack_cached(&dist_info.url, &target, client, net_sem.clone(), cpu_sem.clone(), &p.name, &p.version)
            .await?;
    } else if let Some(source_info) = &p.source {
        if source_info.source_type == "path" {
            copy_local_path_async(&source_info.url, &target, cpu_sem.clone()).await?;
        } else {
            // Handle git sources
            clone_git_async(
                &source_info.url,
                Some(&source_info.reference),
                &target,
                net_sem.clone(),
            )
            .await?;
        }
    } else {
        utils::print_error(&format!("No source/dist for {}", p.name));
        return Err(anyhow::anyhow!("no source/dist for {}", p.name));
    }

    Ok(InstalledPackage {
        name: p.name.clone(),
        version: p.version.clone(),
        path: Utf8PathBuf::from_path_buf(target).unwrap(),
    })
}

async fn copy_local_path_async(src: &str, target: &Path, cpu_sem: Arc<Semaphore>) -> Result<()> {
    let src = PathBuf::from(src);
    let target = target.to_path_buf();
    let permit = cpu_sem.acquire_owned().await?;
    task::spawn_blocking(move || {
        let _p = permit;
        if !src.exists() || !src.is_dir() {
            utils::print_error(&format!("path repo not found: {}", src.display()));
            return Err(anyhow::anyhow!("path repo not found: {}", src.display()));
        }
        let mut options = CopyOptions::new();
        options.overwrite = true;
        options.copy_inside = true;
        copy_dir(&src, &target, &options)?;
        Ok::<(), anyhow::Error>(())
    })
    .await??;
    Ok(())
}

async fn download_and_unpack_cached(
    url: &str,
    target: &Path,
    client: &reqwest::Client,
    net_sem: Arc<Semaphore>,
    cpu_sem: Arc<Semaphore>,
    package_name: &str,
    package_version: &str,
) -> Result<()> {
    let cache_path = get_cached_package_path(package_name, package_version, url);
    
    // Create cache directory if it doesn't exist
    if let Some(parent) = cache_path.parent() {
        fs::create_dir_all(parent).await?;
    }
    
    // Check if cached file exists
    if !cache_path.exists() {
        // Download to cache
        let net_guard = net_sem.acquire_owned().await?;
        let mut cache_file = fs::File::create(&cache_path).await?;
        
        let resp = client.get(url).send().await?.error_for_status()?;
        let mut stream = resp.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let b = chunk?;
            tokio::io::AsyncWriteExt::write_all(&mut cache_file, &b).await?;
        }
        tokio::io::AsyncWriteExt::flush(&mut cache_file).await?;
        drop(net_guard);
    }
    
    // Extract from cache
    let cpu_guard = cpu_sem.acquire_owned().await?;
    let target = target.to_path_buf();
    let cache_path_clone = cache_path.clone();
    task::spawn_blocking(move || -> Result<()> {
        let _cpu = cpu_guard;
        if try_extract_zip(&cache_path_clone, &target).is_ok() {
            return Ok(());
        } else {
            try_extract_tar_gz(&cache_path_clone, &target)?;
        }
        Ok(())
    })
    .await??;
    Ok(())
}

fn try_extract_zip(archive: &std::path::Path, dest: &std::path::Path) -> Result<()> {
    let f = std::fs::File::open(archive)?;
    let mut z = zip::ZipArchive::new(f)?;
    for i in 0..z.len() {
        let mut file = z.by_index(i)?;
        let outpath = dest.join(crate::utils::strip_first_component(file.name()));
        if file.is_dir() {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut out = std::fs::File::create(&outpath)?;
            std::io::copy(&mut file, &mut out)?;
        }
    }
    Ok(())
}

fn try_extract_tar_gz(archive: &std::path::Path, dest: &std::path::Path) -> Result<()> {
    let f = std::fs::File::open(archive)?;
    let reader = std::io::BufReader::new(f);
    let gz = flate2::read::GzDecoder::new(reader);
    let mut ar = tar::Archive::new(gz);
    ar.unpack(dest)?;
    Ok(())
}

async fn clone_git_async(
    url: &str,
    reference: Option<&String>,
    target: &Path,
    net_sem: Arc<Semaphore>,
) -> Result<()> {
    let net_guard = net_sem.acquire_owned().await?;
    let url = url.to_string();
    let target = target.to_path_buf();
    let reference = reference.cloned();

    let handle = task::spawn_blocking(move || -> Result<()> {
        let repo = Repository::clone(&url, &target)?;
        if let Some(r) = reference {
            let obj = repo.revparse_single(&r)?;
            repo.checkout_tree(&obj, None)?;
            repo.set_head_detached(obj.id())?;
        }
        Ok(())
    });

    let res = handle.await?;
    drop(net_guard);
    res
}
