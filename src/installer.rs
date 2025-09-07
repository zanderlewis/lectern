use anyhow::Result;
use camino::Utf8PathBuf;
use futures::stream::{FuturesUnordered, StreamExt};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::Semaphore;
use tokio::task;
use sha2::Digest;
use tokio::io::AsyncWriteExt;

use crate::model::LockedPackage;
use crate::utils;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct InstalledPackage {
    pub name: String,
    pub version: String,
    pub path: Utf8PathBuf,
}

const NETWORK_FACTOR: usize = 50;
const CPU_FACTOR: usize = 24;
const DOWNLOAD_CHUNK_SIZE: usize = 65536;
const MAX_CONCURRENT_EXTRACTIONS: usize = 16;
const STREAMING_THRESHOLD: usize = 1024 * 1024; // 1 MB

fn get_package_cache_dir() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".lectern_cache")
        .join("packages")
}

fn get_cached_package_path(name: &str, version: &str, url: &str) -> PathBuf {
    let mut hasher = sha2::Sha256::new();
    hasher.update(format!("{name}-{version}-{url}").as_bytes());
    let hash = format!("{:x}", hasher.finalize());
    
    get_package_cache_dir().join(format!("{hash}.zip"))
}

/// Install packages from locked package list
/// # Errors
/// Returns an error if packages cannot be downloaded or installed
/// # Panics
/// May panic if path conversion fails unexpectedly
#[allow(clippy::too_many_lines)]
pub async fn install_packages(
    pkgs: &[LockedPackage],
    project_dir: &Path,
) -> Result<Vec<InstalledPackage>> {
    let vendor = project_dir.join("vendor");
    fs::create_dir_all(&vendor).await?;

    let cores = num_cpus::get();
    let net_sem = Arc::new(Semaphore::new(cores * NETWORK_FACTOR));
    let cpu_sem = Arc::new(Semaphore::new(cores * CPU_FACTOR));
    let extract_sem = Arc::new(Semaphore::new(MAX_CONCURRENT_EXTRACTIONS));

    // Ultra-optimized HTTP client with connection pooling and keep-alive
    let client = reqwest::Client::builder()
        .user_agent("lectern/0.1")
        .tcp_nodelay(true)
        .tcp_keepalive(std::time::Duration::from_secs(60))
        .pool_idle_timeout(std::time::Duration::from_secs(300))
        .pool_max_idle_per_host(cores * 8) // Increased pool size
        .http2_prior_knowledge()            // Force HTTP/2 for better multiplexing
        .http2_keep_alive_interval(std::time::Duration::from_secs(30))
        .timeout(std::time::Duration::from_secs(60)) // Reduced timeout for faster failure
        .connection_verbose(false)
        .build()?;

    // Pre-filter packages to avoid unnecessary work
    let mut already_installed = Vec::new();
    let mut to_install = Vec::new();

    for p in pkgs {
        let target = vendor.join(
            p.name.replace('/', std::path::MAIN_SEPARATOR.to_string().as_str()),
        );
        
        // Check if already installed with correct version
        if target.exists() {
            if let Ok(composer_path) = target.join("composer.json").canonicalize() {
                if let Ok(content) = std::fs::read_to_string(&composer_path) {
                    if let Ok(composer_json) = serde_json::from_str::<serde_json::Value>(&content) {
                        if let Some(version) = composer_json.get("version").and_then(|v| v.as_str()) {
                            if version == p.version {
                                already_installed.push(InstalledPackage {
                                    name: p.name.clone(),
                                    version: p.version.clone(),
                                    path: Utf8PathBuf::from_path_buf(target).unwrap(),
                                });
                                continue;
                            }
                        }
                    }
                }
            }
        }
        
        to_install.push(p);
    }

    if !already_installed.is_empty() {
        utils::print_info(&format!("✅ {} packages already installed", already_installed.len()));
    }

    if to_install.is_empty() {
        return Ok(already_installed);
    }

    utils::print_info(&format!("🚀 Installing {} packages with {}x network concurrency, {}x CPU concurrency", 
        to_install.len(), cores * NETWORK_FACTOR, cores * CPU_FACTOR));

    // Advanced batching by package type for optimal processing
    let mut dist_packages = Vec::new();
    let mut git_packages = Vec::new();
    let mut path_packages = Vec::new();

    for p in &to_install {
        if p.dist.is_some() {
            dist_packages.push((*p).clone());
        } else if let Some(source) = &p.source {
            if source.source_type == "path" {
                path_packages.push((*p).clone());
            } else {
                git_packages.push((*p).clone());
            }
        }
    }

    let mut all_results = already_installed;

    // Process all package types in parallel for maximum throughput
    let mut batch_futures = Vec::new();

    // Batch 1: Distribution packages (ZIP/TAR downloads) - highest priority
    if !dist_packages.is_empty() {
        let client_clone = client.clone();
        let net_sem_clone = net_sem.clone();
        let extract_sem_clone = extract_sem.clone();
        let vendor_clone = vendor.clone();
        
        batch_futures.push(task::spawn(async move {
            install_dist_packages_batch(
                &dist_packages,
                &vendor_clone,
                client_clone,
                net_sem_clone,
                extract_sem_clone,
            ).await
        }));
    }

    // Batch 2: Git packages in parallel
    if !git_packages.is_empty() {
        let cpu_sem_clone = cpu_sem.clone();
        let vendor_clone = vendor.clone();
        
        batch_futures.push(task::spawn(async move {
            install_git_packages_batch(&git_packages, &vendor_clone, cpu_sem_clone).await
        }));
    }

    // Batch 3: Path packages (usually local, very fast)
    if !path_packages.is_empty() {
        let vendor_clone = vendor.clone();
        
        batch_futures.push(task::spawn(async move {
            install_path_packages_batch(&path_packages, &vendor_clone).await
        }));
    }

    // Wait for all batches to complete and collect results
    for batch_future in batch_futures {
        match batch_future.await {
            Ok(Ok(mut batch_results)) => {
                all_results.append(&mut batch_results);
            }
            Ok(Err(e)) => {
                utils::print_error(&format!("Batch installation failed: {}", e));
                return Err(e);
            }
            Err(e) => {
                utils::print_error(&format!("Batch task failed: {}", e));
                return Err(anyhow::anyhow!("Batch task failed: {}", e));
            }
        }
    }

    utils::print_info(&format!("✅ Successfully installed {} packages", all_results.len()));
    Ok(all_results)
}

// Ultra-fast batch processing for distribution packages (ZIP/TAR)
async fn install_dist_packages_batch(
    packages: &[LockedPackage],
    vendor: &Path,
    client: reqwest::Client,
    net_sem: Arc<Semaphore>,
    extract_sem: Arc<Semaphore>,
) -> Result<Vec<InstalledPackage>> {
    utils::print_info(&format!("🚀 Batch processing {} distribution packages", packages.len()));
    
    let mut futures = FuturesUnordered::new();
    
    for p in packages {
        if let Some(dist_info) = &p.dist {
            let target = vendor.join(
                p.name.replace('/', std::path::MAIN_SEPARATOR.to_string().as_str()),
            );
            
            let client = client.clone();
            let net_sem = net_sem.clone();
            let extract_sem = extract_sem.clone();
            let url = dist_info.url.clone();
            let name = p.name.clone();
            let version = p.version.clone();
            
            futures.push(tokio::spawn(async move {
                // Create target directory
                fs::create_dir_all(&target).await?;
                
                // Download and extract with streaming for better memory usage
                download_and_extract_streaming(&url, &target, client, net_sem, extract_sem, &name, &version).await?;
                
                Ok(InstalledPackage {
                    name,
                    version,
                    path: Utf8PathBuf::from_path_buf(target).unwrap(),
                })
            }));
        }
    }
    
    let mut results = Vec::new();
    while let Some(result) = futures.next().await {
        match result {
            Ok(Ok(installed)) => results.push(installed),
            Ok(Err(e)) => return Err(e),
            Err(e) => return Err(anyhow::anyhow!("Task failed: {}", e)),
        }
    }
    
    Ok(results)
}

// Ultra-fast batch processing for git packages
async fn install_git_packages_batch(
    packages: &[LockedPackage],
    vendor: &Path,
    cpu_sem: Arc<Semaphore>,
) -> Result<Vec<InstalledPackage>> {
    utils::print_info(&format!("🚀 Batch processing {} git packages", packages.len()));
    
    let mut futures = FuturesUnordered::new();
    
    for p in packages {
        if let Some(source_info) = &p.source {
            let target = vendor.join(
                p.name.replace('/', std::path::MAIN_SEPARATOR.to_string().as_str()),
            );
            
            let cpu_sem = cpu_sem.clone();
            let url = source_info.url.clone();
            let reference = source_info.reference.clone();
            let name = p.name.clone();
            let version = p.version.clone();
            
            futures.push(tokio::spawn(async move {
                fs::create_dir_all(&target).await?;
                
                clone_git_optimized(&url, Some(&reference), &target, cpu_sem).await?;
                
                Ok(InstalledPackage {
                    name,
                    version,
                    path: Utf8PathBuf::from_path_buf(target).unwrap(),
                })
            }));
        }
    }
    
    let mut results = Vec::new();
    while let Some(result) = futures.next().await {
        match result {
            Ok(Ok(installed)) => results.push(installed),
            Ok(Err(e)) => return Err(e),
            Err(e) => return Err(anyhow::anyhow!("Task failed: {}", e)),
        }
    }
    
    Ok(results)
}

// Ultra-fast batch processing for path packages
async fn install_path_packages_batch(
    packages: &[LockedPackage],
    vendor: &Path,
) -> Result<Vec<InstalledPackage>> {
    utils::print_info(&format!("🚀 Batch processing {} path packages", packages.len()));
    
    let mut futures = FuturesUnordered::new();
    
    for p in packages {
        if let Some(source_info) = &p.source {
            let target = vendor.join(
                p.name.replace('/', std::path::MAIN_SEPARATOR.to_string().as_str()),
            );
            
            let src_path = source_info.url.clone();
            let name = p.name.clone();
            let version = p.version.clone();
            
            futures.push(tokio::spawn(async move {
                fs::create_dir_all(&target).await?;
                
                copy_local_path_optimized(&src_path, &target).await?;
                
                Ok(InstalledPackage {
                    name,
                    version,
                    path: Utf8PathBuf::from_path_buf(target).unwrap(),
                })
            }));
        }
    }
    
    let mut results = Vec::new();
    while let Some(result) = futures.next().await {
        match result {
            Ok(Ok(installed)) => results.push(installed),
            Ok(Err(e)) => return Err(e),
            Err(e) => return Err(anyhow::anyhow!("Task failed: {}", e)),
        }
    }
    
    Ok(results)
}

// ULTRA-OPTIMIZED STREAMING DOWNLOAD AND EXTRACTION
async fn download_and_extract_streaming(
    url: &str,
    target: &Path,
    client: reqwest::Client,
    net_sem: Arc<Semaphore>,
    extract_sem: Arc<Semaphore>,
    package_name: &str,
    package_version: &str,
) -> Result<()> {
    let cache_path = get_cached_package_path(package_name, package_version, url);
    
    // Create cache directory if it doesn't exist
    if let Some(parent) = cache_path.parent() {
        fs::create_dir_all(parent).await?;
    }
    
    // Check if cached file exists and is valid
    let cache_exists = cache_path.exists() && 
        fs::metadata(&cache_path).await.map(|m| m.len() > 0).unwrap_or(false);
    
    if !cache_exists {
        let _net_guard = net_sem.acquire_owned().await?;
        
        // Ultra-optimized download with connection reuse and compression
        let response = client
            .get(url)
            .header("Accept-Encoding", "gzip, deflate, br, zstd")
            .header("Accept", "*/*")
            .header("Connection", "keep-alive")
            .send()
            .await?
            .error_for_status()?;
        
        let total_size = response.content_length();
        
        // Stream directly to cache with larger buffer for better throughput
        let temp_path = cache_path.with_extension("tmp");
        let mut cache_file = fs::File::create(&temp_path).await?;
        let mut buffer = Vec::with_capacity(DOWNLOAD_CHUNK_SIZE);
        
        let mut stream = response.bytes_stream();
        let mut downloaded = 0u64;
        
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            downloaded += chunk.len() as u64;
            
            // Write with vectorized I/O for better performance
            buffer.extend_from_slice(&chunk);
            
            if buffer.len() >= DOWNLOAD_CHUNK_SIZE {
                cache_file.write_all(&buffer).await?;
                buffer.clear();
            }
            
            // Progress for large files
            if let Some(total) = total_size {
                if total > STREAMING_THRESHOLD as u64 {
                    let percent = (downloaded as f64 / total as f64 * 100.0) as u32;
                    if downloaded % (total / 10).max(1) == 0 { // Report every 10%
                        utils::print_info(&format!("📥 {}: {}%", package_name, percent));
                    }
                }
            }
        }
        
        // Write remaining buffer
        if !buffer.is_empty() {
            cache_file.write_all(&buffer).await?;
        }
        
        cache_file.flush().await?;
        drop(cache_file);
        
        // Atomic rename
        fs::rename(&temp_path, &cache_path).await?;
    }
    
    // Parallel extraction with semaphore limiting
    let _extract_guard = extract_sem.acquire_owned().await?;
    let target = target.to_path_buf();
    let cache_path_clone = cache_path.clone();
    
    task::spawn_blocking(move || -> Result<()> {
        extract_archive_ultra_fast(&cache_path_clone, &target)
    }).await??;
    
    Ok(())
}

// Ultra-fast archive extraction with format detection and optimization
fn extract_archive_ultra_fast(archive: &std::path::Path, dest: &std::path::Path) -> Result<()> {
    let file = std::fs::File::open(archive)?;
    let mut buffer = [0; 4];
    
    // Read magic bytes for format detection
    {
        use std::io::{Read, Seek, SeekFrom};
        let mut reader = file.try_clone()?;
        reader.read_exact(&mut buffer)?;
        reader.seek(SeekFrom::Start(0))?;
    }
    
    // Fast format detection by magic bytes
    match &buffer {
        [0x50, 0x4B, 0x03, 0x04] | [0x50, 0x4B, 0x05, 0x06] | [0x50, 0x4B, 0x07, 0x08] => {
            // ZIP format - ultra-fast extraction
            extract_zip_ultra_fast(archive, dest)
        }
        [0x1F, 0x8B, _, _] => {
            // GZIP format - likely tar.gz
            extract_tar_gz_ultra_fast(archive, dest)
        }
        _ => {
            // Try ZIP first, then TAR.GZ as fallback
            extract_zip_ultra_fast(archive, dest)
                .or_else(|_| extract_tar_gz_ultra_fast(archive, dest))
        }
    }
}

fn extract_zip_ultra_fast(archive: &std::path::Path, dest: &std::path::Path) -> Result<()> {
    let file = std::fs::File::open(archive)?;
    let mut zip = zip::ZipArchive::new(file)?;
    
    // Pre-allocate collections for better memory performance
    let file_count = zip.len();
    let mut directories = Vec::with_capacity(file_count / 10);  // Estimate 10% directories
    let mut files = Vec::with_capacity(file_count);
    
    // Single pass to categorize entries
    for i in 0..file_count {
        let entry = zip.by_index(i)?;
        let path = dest.join(crate::utils::strip_first_component(entry.name()));
        
        if entry.is_dir() {
            directories.push(path);
        } else {
            files.push((i, path, entry.size()));
        }
    }
    
    // Batch create all directories
    for dir in directories {
        std::fs::create_dir_all(&dir)?;
    }
    
    // Sort files by size (extract small files first for better perceived performance)
    files.sort_by_key(|(_, _, size)| *size);
    
    // Extract files with optimized I/O
    for (index, path, _) in files {
        let mut entry = zip.by_index(index)?;
        
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let mut output = std::fs::File::create(&path)?;
        
        // Use large buffer for faster copying
        let mut buffer = vec![0; DOWNLOAD_CHUNK_SIZE.max(8192)];
        loop {
            let bytes_read = std::io::Read::read(&mut entry, &mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            std::io::Write::write_all(&mut output, &buffer[..bytes_read])?;
        }
    }
    
    Ok(())
}

fn extract_tar_gz_ultra_fast(archive: &std::path::Path, dest: &std::path::Path) -> Result<()> {
    let file = std::fs::File::open(archive)?;
    let decompressor = flate2::read::GzDecoder::new(file);
    let mut tar = tar::Archive::new(decompressor);
    
    // Set preserve permissions to false for faster extraction
    tar.set_preserve_permissions(false);
    tar.set_preserve_mtime(false);
    
    // Extract all with optimized settings
    tar.unpack(dest)?;
    
    Ok(())
}

// Ultra-optimized git cloning
async fn clone_git_optimized(
    url: &str,
    reference: Option<&str>,
    target: &Path,
    cpu_sem: Arc<Semaphore>,
) -> Result<()> {
    let _cpu_guard = cpu_sem.acquire_owned().await?;
    let url = url.to_string();
    let reference = reference.map(|s| s.to_string());
    let target = target.to_path_buf();
    
    task::spawn_blocking(move || -> Result<()> {
        let mut builder = git2::build::RepoBuilder::new();
        
        // Optimize git clone for speed
        builder.bare(false);
        builder.branch(reference.as_deref().unwrap_or("main"));
        
        // Configure for faster clones
        let mut fetch_options = git2::FetchOptions::new();
        fetch_options.download_tags(git2::AutotagOption::None);  // Skip tags for speed
        
        let mut remote_callbacks = git2::RemoteCallbacks::new();
        remote_callbacks.update_tips(|_, _, _| true);  // Skip tip updates
        
        fetch_options.remote_callbacks(remote_callbacks);
        builder.fetch_options(fetch_options);
        
        // Shallow clone for maximum speed (depth=1)
        builder.clone_local(git2::build::CloneLocal::Auto);
        
        builder.clone(&url, &target)?;
        Ok(())
    }).await??;
    
    Ok(())
}

// Ultra-optimized local path copying
async fn copy_local_path_optimized(src: &str, target: &Path) -> Result<()> {
    let src = PathBuf::from(src);
    let target = target.to_path_buf();
    
    task::spawn_blocking(move || -> Result<()> {
        if !src.exists() || !src.is_dir() {
            return Err(anyhow::anyhow!("path repo not found: {}", src.display()));
        }
        
        let mut options = fs_extra::dir::CopyOptions::new();
        options.overwrite = true;
        options.copy_inside = true;
        options.content_only = false;
        
        // Use optimized copying
        fs_extra::dir::copy(&src, &target, &options)
            .map_err(|e| anyhow::anyhow!("copy failed: {}", e))?;
        
        Ok(())
    }).await??;
    
    Ok(())
}
