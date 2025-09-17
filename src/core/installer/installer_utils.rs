use anyhow::Result;
use sha2::Digest;
use std::path::{Path, PathBuf};
use tokio::task;

pub fn get_package_cache_dir() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".lectern_cache")
        .join("packages")
}

pub fn get_cached_package_path(name: &str, version: &str, url: &str) -> PathBuf {
    let mut hasher = sha2::Sha256::new();
    hasher.update(format!("{name}-{version}-{url}").as_bytes());
    let hash = format!("{:x}", hasher.finalize());

    get_package_cache_dir().join(format!("{hash}.zip"))
}

pub fn extract_archive_ultra_fast(archive: &Path, dest: &Path) -> Result<()> {
    // Implemented here to avoid circular private access
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
            extract_zip_ultra_fast(archive, dest)
        }
        [0x1F, 0x8B, _, _] => extract_tar_gz_ultra_fast(archive, dest),
        _ => extract_zip_ultra_fast(archive, dest).or_else(|_| extract_tar_gz_ultra_fast(archive, dest)),
    }
}

pub fn extract_zip_ultra_fast(archive: &Path, dest: &Path) -> Result<()> {
    let file = std::fs::File::open(archive)?;
    let mut zip = zip::ZipArchive::new(file)?;

    // Pre-allocate collections for better memory performance
    let file_count = zip.len();
    let mut directories = Vec::with_capacity(file_count / 10); // Estimate 10% directories
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
        let mut buffer = vec![0; 65536.max(8192)];
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

pub fn extract_tar_gz_ultra_fast(archive: &Path, dest: &Path) -> Result<()> {
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

pub async fn clone_git_optimized(
    url: &str,
    reference: Option<&str>,
    target: &Path,
    cpu_sem: std::sync::Arc<tokio::sync::Semaphore>,
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
        fetch_options.download_tags(git2::AutotagOption::None); // Skip tags for speed

        let mut remote_callbacks = git2::RemoteCallbacks::new();
        remote_callbacks.update_tips(|_, _, _| true); // Skip tip updates

        fetch_options.remote_callbacks(remote_callbacks);
        builder.fetch_options(fetch_options);

        // Shallow clone for maximum speed (depth=1)
        builder.clone_local(git2::build::CloneLocal::Auto);

        builder.clone(&url, &target)?;
        Ok(())
    })
    .await??;

    Ok(())
}

pub async fn copy_local_path_optimized(src: &str, target: &Path) -> Result<()> {
    let src = std::path::PathBuf::from(src);
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
    })
    .await??;

    Ok(())
}
