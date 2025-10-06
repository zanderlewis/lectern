use crate::io::read_lock;
use crate::resolver::fetch_package_info;
use crate::utils::is_prerelease_version;
use crate::utils::{print_error, print_info, print_success};
use anyhow::Result;
use futures::stream::{self, StreamExt};
use semver::Version;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Check for outdated packages with incremental updates
/// # Errors
/// Returns an error if the lock file cannot be read or packages cannot be fetched
/// # Panics
/// May panic if version parsing fails unexpectedly
pub async fn check_outdated_packages(working_dir: &Path, quiet: bool) -> Result<()> {
    if !quiet {
        print_info("🔍 Checking for outdated packages...");
    }

    let lock_path = working_dir.join("composer.lock");

    if !lock_path.exists() {
        print_error("❌ No composer.lock found. Run 'lectern install' first.");
        return Ok(());
    }

    let lock = read_lock(&lock_path)?;
    let total_packages = lock.packages.len() + lock.packages_dev.len();

    if total_packages == 0 {
        if !quiet {
            print_info("📦 No packages installed.");
        }
        return Ok(());
    }

    let mut package_names: Vec<String> = lock.packages.iter().map(|p| p.name.clone()).collect();
    package_names.extend(lock.packages_dev.iter().map(|p| p.name.clone()));

    // Use faster in-memory cache instead of file-based cache
    // Batch API requests for package info with higher concurrency
    let package_info_map = Arc::new(Mutex::new(HashMap::new()));
    let concurrency_limit = 50;

    stream::iter(package_names.clone())
        .map(|package_name| {
            let package_info_map = Arc::clone(&package_info_map);
            async move {
                // fetch_package_info already uses cache internally
                if let Ok(result) = fetch_package_info(&package_name).await {
                    let mut map = package_info_map.lock().unwrap();
                    map.insert(package_name, result);
                }
            }
        })
        .buffer_unordered(concurrency_limit)
        .for_each(|_| async {})
        .await;

    let package_info_map = Arc::try_unwrap(package_info_map)
        .unwrap()
        .into_inner()
        .unwrap();

    let mut outdated_count = 0;
    let mut table_rows = Vec::new();

    for package_name in package_names.clone() {
        // Look in both regular and dev packages
        let locked_pkg = lock
            .packages
            .iter()
            .find(|p| p.name == package_name)
            .or_else(|| lock.packages_dev.iter().find(|p| p.name == package_name));

        if let Some(locked_pkg) = locked_pkg {
            if let Some(package_info) = package_info_map.get(&package_name) {
                if let Some(versions) = &package_info.package.versions {
                    // Find the latest stable version
                    let mut latest_version = None;
                    let mut latest_parsed: Option<Version> = None;

                    // Parse the current version
                    let current_version_str = locked_pkg.version.trim_start_matches('v');
                    let current_parsed = Version::parse(current_version_str).ok();

                    for version_str in versions.keys() {
                        // Skip dev versions and pre-releases for "latest" comparison
                        if is_prerelease_version(version_str.as_str()) {
                            continue;
                        }

                        // Try to parse the version
                        let clean_version = version_str.trim_start_matches('v');
                        if let Ok(parsed_version) = Version::parse(clean_version) {
                            if latest_parsed.is_none()
                                || parsed_version > *latest_parsed.as_ref().unwrap()
                            {
                                latest_parsed = Some(parsed_version);
                                latest_version = Some(version_str.clone());
                            }
                        }
                    }

                    // Check if the latest version is newer than current
                    if let (Some(current), Some(latest_ver), Some(latest_str)) =
                        (current_parsed, latest_parsed, latest_version)
                    {
                        if latest_ver > current {
                            outdated_count += 1;
                            table_rows.push((
                                package_name.clone(),
                                locked_pkg.version.clone(),
                                latest_str,
                                package_info.package.description.clone().unwrap_or_default(),
                            ));
                        }
                    }
                }
            }
        }
    }

    if outdated_count == 0 {
        if !quiet {
            print_success("✅ All packages are up to date!");
        }
    } else if !quiet {
        println!("\n📊 Outdated Packages ({outdated_count} found):");
        println!(
            "{:<30} {:<15} {:<15} Description",
            "Package", "Current", "Latest"
        );
        println!("{}", "-".repeat(100));

        for (name, current, latest, desc) in table_rows {
            let short_desc = if desc.len() > 30 {
                format!("{}...", &desc[..27])
            } else {
                desc
            };
            println!("{name:<30} {current:<15} {latest:<15} {short_desc}");
        }

        println!("\nRun 'lectern update' to update packages.");
    }

    Ok(())
}
