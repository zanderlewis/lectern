use crate::io::read_lock;
use crate::resolver::{fetch_multiple_package_info, fetch_package_info, search_packagist};
use crate::utils::{print_error, print_info, print_success};
use anyhow::Result;
use semver::Version;
use std::path::Path;

/// Check for outdated packages
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

    if lock.packages.is_empty() {
        if !quiet {
            print_info("📦 No packages installed.");
        }
        return Ok(());
    }

    // Collect package names for bulk fetching
    let package_names: Vec<String> = lock.packages.iter().map(|p| p.name.clone()).collect();

    // Fetch package info for all packages concurrently
    let package_infos = fetch_multiple_package_info(&package_names).await?;

    let mut outdated_count = 0;
    let mut table_rows = Vec::new();

    for (package_name, package_info_opt) in package_infos {
        if let Some(locked_pkg) = lock.packages.iter().find(|p| p.name == package_name) {
            if let Some(package_info) = package_info_opt {
                if let Some(versions) = &package_info.package.versions {
                    // Find the latest stable version
                    let mut latest_version = None;
                    let mut latest_parsed: Option<Version> = None;

                    // Parse the current version
                    let current_version_str = locked_pkg.version.trim_start_matches('v');
                    let current_parsed = Version::parse(current_version_str).ok();

                    for version_str in versions.keys() {
                        // Skip dev versions and pre-releases for "latest" comparison
                        if version_str.contains("dev")
                            || version_str.contains("alpha")
                            || version_str.contains("beta")
                            || version_str.contains("rc")
                        {
                            continue;
                        }

                        // Try to parse the version
                        let clean_version = version_str.trim_start_matches('v');
                        if let Ok(parsed_version) = Version::parse(clean_version) {
                            if latest_parsed.is_none() || parsed_version > *latest_parsed.as_ref().unwrap() {
                                latest_parsed = Some(parsed_version);
                                latest_version = Some(version_str.clone());
                            }
                        }
                    }

                    // Check if the latest version is newer than current
                    if let (Some(current), Some(latest_ver), Some(latest_str)) = 
                        (current_parsed, latest_parsed, latest_version) {
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

/// Show licenses of all dependencies
pub async fn show_dependency_licenses(working_dir: &Path, quiet: bool) -> Result<()> {
    if !quiet {
        print_info("📜 Fetching license information...");
    }

    let lock_path = working_dir.join("composer.lock");

    if !lock_path.exists() {
        print_error("❌ No composer.lock found. Run 'lectern install' first.");
        return Ok(());
    }

    let lock = read_lock(&lock_path)?;

    if lock.packages.is_empty() {
        if !quiet {
            print_info("📦 No packages installed.");
        }
        return Ok(());
    }

    // Collect package names for bulk fetching
    let package_names: Vec<String> = lock.packages.iter().map(|p| p.name.clone()).collect();

    // Fetch package info for all packages concurrently
    let package_infos = fetch_multiple_package_info(&package_names).await?;

    let mut table_rows = Vec::new();

    for (package_name, package_info_opt) in package_infos {
        if let Some(locked_pkg) = lock.packages.iter().find(|p| p.name == package_name) {
            let mut license_info = "Unknown".to_string();

            if let Some(package_info) = package_info_opt {
                if let Some(versions) = &package_info.package.versions {
                    // Look for license in the current version
                    if let Some(version_details) = versions.get(&locked_pkg.version) {
                        if let Some(licenses) = &version_details.license {
                            license_info = licenses.join(", ");
                        }
                    }

                    // Fallback: look in any version
                    if license_info == "Unknown" {
                        for details in versions.values() {
                            if let Some(licenses) = &details.license {
                                license_info = licenses.join(", ");
                                break;
                            }
                        }
                    }
                }
            }

            table_rows.push((package_name, locked_pkg.version.clone(), license_info));
        }
    }

    if !quiet {
        println!("\n📜 Package Licenses:");
        println!("{:<40} {:<15} License", "Package", "Version");
        println!("{}", "-".repeat(80));

        table_rows.sort_by(|a, b| a.0.cmp(&b.0));
        for (name, version, license) in table_rows {
            println!("{name:<40} {version:<15} {license}");
        }
    }

    Ok(())
}

/// Show status of all dependencies
pub async fn show_dependency_status(working_dir: &Path) -> Result<()> {
    print_info("📊 Checking dependency status...");

    let lock_path = working_dir.join("composer.lock");

    if !lock_path.exists() {
        print_error("❌ No composer.lock found. Run 'lectern install' first.");
        return Ok(());
    }

    let lock = read_lock(&lock_path)?;

    if !lock.packages.is_empty() {
        println!("\n📦 Installed Packages ({} total):", lock.packages.len());
        println!("{:<40} {:<15} Type", "Package", "Version");
        println!("{}", "-".repeat(70));

        for pkg in &lock.packages {
            let pkg_type = if pkg.name.starts_with("symfony/") {
                "symfony"
            } else if pkg.name.starts_with("laravel/") {
                "laravel"
            } else if pkg.name.starts_with("phpunit/") {
                "testing"
            } else if pkg.name.starts_with("psr/") {
                "psr"
            } else {
                "library"
            };

            println!("{:<40} {:<15} {}", pkg.name, pkg.version, pkg_type);
        }

        print_success(&format!("✅ {} packages installed", lock.packages.len()));
    } else {
        print_info("📦 No packages installed.");
    }

    Ok(())
}

/// Search for packages on Packagist
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
            .map(|d| d.to_string())
            .unwrap_or_else(|| "N/A".to_string());

        println!("{:<30} {:<50} {}", result.name, short_desc, downloads);
    }

    Ok(())
}

/// Show detailed information about a specific package
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
