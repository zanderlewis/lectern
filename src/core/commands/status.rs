use crate::io::read_lock;
use crate::utils::{print_error, print_info, print_success};
use anyhow::Result;
use std::path::Path;

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
