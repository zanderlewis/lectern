use crate::io::{read_lock, write_cache, read_cache};
use crate::resolver::{fetch_package_info, search_packagist};
use crate::utils::{print_error, print_info, print_success};
use anyhow::Result;
use futures::stream::{self, StreamExt};
use semver::Version;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use crate::utils::is_prerelease_version;

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

    // Load cached state
    let cache_path = Path::new("cache.json");
    let mut cached_versions: HashMap<String, String> = if cache_path.exists() {
        read_cache(&cache_path).unwrap_or_default()
    } else {
        HashMap::new()
    };

    // Determine packages to fetch
    let packages_to_fetch: Vec<String> = package_names
        .iter()
        .filter(|name| {
            if let Some(locked_pkg) = lock.packages.iter().find(|p| &p.name == *name) {
                cached_versions.get(&**name) != Some(&locked_pkg.version)
            } else {
                true
            }
        })
        .cloned()
        .collect();

    // Batch API requests for package info
    let package_info_map = Arc::new(Mutex::new(HashMap::new()));
    let concurrency_limit = 20;
    stream::iter(packages_to_fetch.clone())
        .map(|package_name| {
            let package_info_map = Arc::clone(&package_info_map);
            async move {
                if let Ok(result) = fetch_package_info(&package_name).await {
                    let mut map = package_info_map.lock().unwrap();
                    map.insert(package_name, result);
                }
            }
        })
        .buffer_unordered(concurrency_limit)
        .for_each(|_| async {})
        .await;

    let package_info_map = Arc::try_unwrap(package_info_map).unwrap().into_inner().unwrap();

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

    // Update cache
    for package_name in package_names {
        if let Some(locked_pkg) = lock.packages.iter().find(|p| p.name == package_name) {
            cached_versions.insert(package_name, locked_pkg.version.clone());
        }
    }
    write_cache(&cache_path, &cached_versions)?;

    if outdated_count == 0 {
        if !quiet {
            print_success("✅ All packages are up to date!");
        }
    } else if !quiet {
        println!("\n📊 Outdated Packages ({} found):", outdated_count);
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
            println!("{:<30} {:<15} {:<15} {}", name, current, latest, short_desc);
        }

        println!("\nRun 'lectern update' to update packages.");
    }

    Ok(())
}

/// Show licenses of all dependencies
/// # Errors
/// Returns an error if the lock file cannot be read
pub async fn show_dependency_licenses(working_dir: &Path, quiet: bool) -> Result<()> {
    if !quiet {
        print_info("📜 Reading license information from lock file...");
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

    let mut table_rows = Vec::new();

    // Process regular packages
    for pkg in &lock.packages {
        let license_info = pkg
            .license
            .as_ref()
            .map_or_else(|| "Unknown".to_string(), |licenses| licenses.join(", "));

        table_rows.push((pkg.name.clone(), pkg.version.clone(), license_info));
    }

    // Process dev packages
    for pkg in &lock.packages_dev {
        let license_info = pkg
            .license
            .as_ref()
            .map_or_else(|| "Unknown".to_string(), |licenses| licenses.join(", "));

        table_rows.push((pkg.name.clone(), pkg.version.clone(), license_info));
    }

    if !quiet {
        println!("\n📜 Package Licenses:");
        println!("{:<40} {:<15} License", "Package", "Version");
        println!("{}", "-".repeat(80));

        table_rows.sort_by(|a, b| a.0.cmp(&b.0));
        let package_count = table_rows.len();

        for (name, version, license) in table_rows {
            println!("{name:<40} {version:<15} {license}");
        }

        print_success(&format!("📊 Listed licenses for {package_count} packages"));
    }

    Ok(())
}

/// Show status of all dependencies
/// # Errors
/// Returns an error if the lock file cannot be read
pub async fn show_dependency_status(working_dir: &Path) -> Result<()> {
    print_info("📊 Checking dependency status...");

    let lock_path = working_dir.join("composer.lock");

    if !lock_path.exists() {
        print_error("❌ No composer.lock found. Run 'lectern install' first.");
        return Ok(());
    }

    let lock = read_lock(&lock_path)?;

    let total_packages = lock.packages.len() + lock.packages_dev.len();

    if total_packages > 0 {
        println!("\n📦 Installed Packages ({total_packages} total):");
        println!("{:<40} {:<15} Type", "Package", "Version");
        println!("{}", "-".repeat(70));

        for pkg in &lock.packages {
            println!("{:<40} {:<15} (regular)", pkg.name, pkg.version);
        }

        // Show dev packages
        for pkg in &lock.packages_dev {
            println!("{:<40} {:<15} (dev)", pkg.name, pkg.version);
        }

        print_success(&format!("✅ {total_packages} packages installed"));
    } else {
        print_info("📦 No packages installed.");
    }

    Ok(())
}

/// Search for packages on Packagist
/// # Errors
/// Returns an error if the search request fails
pub async fn search_packages(terms: &[String], _working_dir: &Path) -> Result<()> {
    if terms.is_empty() {
        print_error("❌ Please provide search terms");
        return Ok(());
    }

    print_info(&format!("🔍 Searching for: {}", terms.join(" ")));

    let results = search_packagist(terms).await?;

    if results.is_empty() {
        print_info("📦 No packages found matching your search.");
        return Ok(());
    }

    println!("\n🔍 Search Results ({} found):", results.len());
    println!("{:<30} {:<50} Downloads", "Package", "Description");
    println!("{}", "-".repeat(100));

    for result in results.iter().take(15) {
        let desc = result.description.as_deref().unwrap_or("No description");
        let short_desc = if desc.len() > 47 {
            format!("{}...", &desc[..44])
        } else {
            desc.to_string()
        };

        let downloads = result
            .downloads
            .map_or_else(|| "N/A".to_string(), |d| d.to_string());

        println!("{:<30} {:<50} {}", result.name, short_desc, downloads);
    }

    Ok(())
}

/// Show detailed information about a specific package
/// # Errors
/// Returns an error if the package information cannot be fetched
pub async fn show_package_details(package: &str, _working_dir: &Path) -> Result<()> {
    print_info(&format!("📦 Fetching details for: {package}"));

    let package_info = fetch_package_info(package).await?;

    println!("\n📦 Package: {}", package_info.package.name);

    if let Some(desc) = &package_info.package.description {
        println!("📝 Description: {desc}");
    }

    if let Some(package_type) = &package_info.package.package_type {
        println!("🏷️  Type: {package_type}");
    }

    if let Some(repo) = &package_info.package.repository {
        println!("🔗 Repository: {repo}");
    }

    if let Some(downloads) = &package_info.package.downloads {
        if let Some(total) = downloads.total {
            println!("📈 Total Downloads: {total}");
        }
        if let Some(monthly) = downloads.monthly {
            println!("📅 Monthly Downloads: {monthly}");
        }
    }

    if let Some(favers) = package_info.package.favers {
        println!("⭐ Stars: {favers}");
    }

    if let Some(maintainers) = &package_info.package.maintainers {
        println!("👥 Maintainers:");
        for maintainer in maintainers.iter().take(5) {
            println!("   • {}", maintainer.name);
        }
    }

    if let Some(versions) = &package_info.package.versions {
        println!("📋 Recent Versions:");
        let mut version_list: Vec<_> = versions.keys().collect();
        version_list.sort();
        version_list.reverse();

        for version in version_list.iter().take(10) {
            if let Some(version_info) = versions.get(*version) {
                let time = version_info.time.as_deref().unwrap_or("Unknown");
                println!("   • {version} ({time})");
            }
        }
    }

    Ok(())
}
