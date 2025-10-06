use crate::resolver::fetch_package_info;
use crate::utils::print_info;
use anyhow::Result;
use std::path::Path;

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
