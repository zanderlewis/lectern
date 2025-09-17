use anyhow::Result;
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::sync::Semaphore;
use tokio::task;
use futures::StreamExt;

use crate::core::installer::installer_utils as inst_utils;

const DOWNLOAD_CHUNK_SIZE: usize = 65536;
const STREAMING_THRESHOLD: usize = 1024 * 1024; // 1 MB

pub fn get_cached_package_path(name: &str, version: &str, url: &str) -> std::path::PathBuf {
    inst_utils::get_cached_package_path(name, version, url)
}

pub async fn download_and_extract_streaming(
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
    let cache_exists = cache_path.exists()
        && fs::metadata(&cache_path)
            .await
            .map(|m| m.len() > 0)
            .unwrap_or(false);

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
                    if downloaded % (total / 10).max(1) == 0 {
                        // Report every 10%
                        crate::core::utils::print_info(&format!("📥 {package_name}: {percent}%"));
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

    task::spawn_blocking(move || -> Result<()> { inst_utils::extract_archive_ultra_fast(&cache_path_clone, &target) })
        .await??;

    Ok(())
}
