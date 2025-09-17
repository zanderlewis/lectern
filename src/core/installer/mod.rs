// installer submodules grouped under src/core/installer/
pub mod installer_utils;
pub mod installer_io;

// Re-export commonly used items at crate::core::installer::*
pub use installer_utils as inst_utils;
pub use installer_io::*;

use anyhow::Result;
use camino::Utf8PathBuf;
use futures::stream::{FuturesUnordered, StreamExt};
// sha2::Digest moved to installer_utils when needed
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::Semaphore;
use tokio::task;

use crate::models::model::LockedPackage;
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
const MAX_CONCURRENT_EXTRACTIONS: usize = 16;

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
		.http2_prior_knowledge() // Force HTTP/2 for better multiplexing
		.http2_keep_alive_interval(std::time::Duration::from_secs(30))
		.timeout(std::time::Duration::from_secs(60)) // Reduced timeout for faster failure
		.connection_verbose(false)
		.build()?;

	// Pre-filter packages to avoid unnecessary work
	let mut already_installed = Vec::new();
	let mut to_install = Vec::new();

	for p in pkgs {
		let target = vendor.join(
			p.name
				.replace('/', std::path::MAIN_SEPARATOR.to_string().as_str()),
		);

		// Check if already installed with correct version
		if target.exists() {
			if let Ok(composer_path) = target.join("composer.json").canonicalize() {
				if let Ok(content) = std::fs::read_to_string(&composer_path) {
					if let Ok(composer_json) = serde_json::from_str::<serde_json::Value>(&content) {
						if let Some(version) = composer_json.get("version").and_then(|v| v.as_str())
						{
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
		utils::print_info(&format!(
			"✅ {} packages already installed",
			already_installed.len()
		));
	}

	if to_install.is_empty() {
		return Ok(already_installed);
	}

	utils::print_info(&format!(
		"🚀 Installing {} packages with {}x network concurrency, {}x CPU concurrency",
		to_install.len(),
		cores * NETWORK_FACTOR,
		cores * CPU_FACTOR
	));

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
			)
			.await
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
				utils::print_error(&format!("Batch installation failed: {e}"));
				return Err(e);
			}
			Err(e) => {
				utils::print_error(&format!("Batch task failed: {e}"));
				return Err(anyhow::anyhow!("Batch task failed: {}", e));
			}
		}
	}

	utils::print_info(&format!(
		"✅ Successfully installed {} packages",
		all_results.len()
	));
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
	utils::print_info(&format!(
		"🚀 Batch processing {} distribution packages",
		packages.len()
	));

	let mut futures = FuturesUnordered::new();

	for p in packages {
		if let Some(dist_info) = &p.dist {
			let target = vendor.join(
				p.name
					.replace('/', std::path::MAIN_SEPARATOR.to_string().as_str()),
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
				installer_io::download_and_extract_streaming(
					&url,
					&target,
					client,
					net_sem,
					extract_sem,
					&name,
					&version,
				)
				.await?;

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
	utils::print_info(&format!(
		"🚀 Batch processing {} git packages",
		packages.len()
	));

	let mut futures = FuturesUnordered::new();

	for p in packages {
		if let Some(source_info) = &p.source {
			let target = vendor.join(
				p.name
					.replace('/', std::path::MAIN_SEPARATOR.to_string().as_str()),
			);

			let cpu_sem = cpu_sem.clone();
			let url = source_info.url.clone();
			let reference = source_info.reference.clone();
			let name = p.name.clone();
			let version = p.version.clone();

			futures.push(tokio::spawn(async move {
				fs::create_dir_all(&target).await?;

				inst_utils::clone_git_optimized(&url, Some(&reference), &target, cpu_sem).await?;

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
	utils::print_info(&format!(
		"🚀 Batch processing {} path packages",
		packages.len()
	));

	let mut futures = FuturesUnordered::new();

	for p in packages {
		if let Some(source_info) = &p.source {
			let target = vendor.join(
				p.name
					.replace('/', std::path::MAIN_SEPARATOR.to_string().as_str()),
			);

			let src_path = source_info.url.clone();
			let name = p.name.clone();
			let version = p.version.clone();

			futures.push(tokio::spawn(async move {
				fs::create_dir_all(&target).await?;

				inst_utils::copy_local_path_optimized(&src_path, &target).await?;

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

