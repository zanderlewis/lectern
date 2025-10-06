use crate::io::read_lock;
use crate::resolver::fetch_packagist_versions_bulk;
use crate::utils::is_prerelease_version;
use crate::utils::{print_error, print_info, print_success};
use anyhow::Result;
use semver::Version;
use std::path::Path;

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

    // Collect only non-platform packages to check
    let mut package_names: Vec<String> = Vec::new();
    for pkg in lock.packages.iter().chain(lock.packages_dev.iter()) {
        // Skip platform packages (php, ext-*, lib-*, etc.) - they can't be outdated
        if pkg.name.starts_with("php")
            || pkg.name.starts_with("ext-")
            || pkg.name.starts_with("lib-")
            || pkg.name == "hhvm"
        {
            continue;
        }
        package_names.push(pkg.name.clone());
    }

    if package_names.is_empty() {
        if !quiet {
            print_success("✅ All packages are up to date!");
        }
        return Ok(());
    }

    // Use optimized bulk P2 API endpoint for much better performance
    // This fetches only version metadata, not full package info
    let versions_map = fetch_packagist_versions_bulk(&package_names).await?;

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
            if let Some(versions) = versions_map.get(&package_name) {
                // Find the latest stable version with early termination
                let mut latest_version = None;
                let mut latest_parsed: Option<Version> = None;

                // Parse the current version
                let current_version_str = locked_pkg.version.trim_start_matches('v');
                let current_parsed = Version::parse(current_version_str).ok();

                // Sort versions in descending order and stop at first stable version
                let mut version_list: Vec<_> = versions.iter().collect();
                version_list.sort_by(|a, b| {
                    let a_clean = a.version.trim_start_matches('v');
                    let b_clean = b.version.trim_start_matches('v');
                    
                    match (Version::parse(a_clean), Version::parse(b_clean)) {
                        (Ok(va), Ok(vb)) => vb.cmp(&va), // Descending order
                        _ => std::cmp::Ordering::Equal,
                    }
                });

                // Find the latest stable version (early termination)
                for version_data in version_list {
                    let version_str = &version_data.version;
                    
                    // Skip dev versions and pre-releases for "latest" comparison
                    if is_prerelease_version(version_str.as_str()) {
                        continue;
                    }

                    // Try to parse the version
                    let clean_version = version_str.trim_start_matches('v');
                    if let Ok(parsed_version) = Version::parse(clean_version) {
                        // Since we're sorted, this is the latest stable version
                        latest_parsed = Some(parsed_version);
                        latest_version = Some(version_str.clone());
                        break; // Early termination - found latest stable
                    }
                }

                // Check if the latest version is newer than current
                if let (Some(current), Some(latest_ver), Some(latest_str)) =
                    (current_parsed, latest_parsed, latest_version)
                {
                    if latest_ver > current {
                        outdated_count += 1;
                        
                        // Get description from version data if available
                        let description = versions
                            .iter()
                            .find(|v| v.version == latest_str)
                            .and_then(|v| v.other.get("description"))
                            .and_then(|d| d.as_str())
                            .unwrap_or("")
                            .to_string();
                        
                        table_rows.push((
                            package_name.clone(),
                            locked_pkg.version.clone(),
                            latest_str,
                            description,
                        ));
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
