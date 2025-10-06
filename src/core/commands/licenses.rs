use crate::io::read_lock;
use crate::utils::{print_error, print_info, print_success};
use anyhow::Result;
use std::path::Path;

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
