use crate::io::read_lock;
use crate::utils::{print_error, print_info, print_step};
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

/// Show suggested packages
pub async fn show_suggests(working_dir: &Path) -> Result<()> {
    print_step("🔍 Checking for suggested packages...");

    let lock_path = working_dir.join("composer.lock");
    if !lock_path.exists() {
        print_error("❌ No composer.lock found. Run 'lectern install' first.");
        return Ok(());
    }

    let lock = read_lock(&lock_path)?;
    let mut suggestions: HashMap<String, Vec<(String, String)>> = HashMap::new();

    // Collect suggestions from all packages
    for pkg in lock.packages.iter().chain(lock.packages_dev.iter()) {
        if let Some(suggest) = &pkg.suggest {
            for (suggested_pkg, reason) in suggest.iter() {
                suggestions
                    .entry(suggested_pkg.clone())
                    .or_default()
                    .push((pkg.name.clone(), reason.clone()));
            }
        }
    }

    if suggestions.is_empty() {
        print_info("No package suggestions found");
    } else {
        println!("\n💡 Suggested packages:");
        for (pkg, reasons) in suggestions.iter() {
            println!("\n  📦 {pkg}");
            for (from_pkg, reason) in reasons {
                println!("    • From {from_pkg}: {reason}");
            }
        }
        println!("\nRun 'lectern require <package>' to install any of these packages.");
    }

    Ok(())
}
