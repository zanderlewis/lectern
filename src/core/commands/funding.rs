use crate::io::read_lock;
use crate::utils::{print_error, print_info, print_step};
use anyhow::Result;
use std::path::Path;

/// Show funding information
pub async fn show_funding(working_dir: &Path) -> Result<()> {
    print_step("💰 Checking for funding information...");

    let lock_path = working_dir.join("composer.lock");
    if !lock_path.exists() {
        print_error("❌ No composer.lock found. Run 'lectern install' first.");
        return Ok(());
    }

    let lock = read_lock(&lock_path)?;
    let mut has_funding = false;

    println!("\n💰 Packages with funding information:");

    for pkg in lock.packages.iter().chain(lock.packages_dev.iter()) {
        if let Some(funding) = &pkg.funding {
            if !funding.is_empty() {
                has_funding = true;
                println!("\n  📦 {}", pkg.name);
                for fund in funding {
                    if let Some(fund_type) = fund.get("type").and_then(|v| v.as_str()) {
                        if let Some(url) = fund.get("url").and_then(|v| v.as_str()) {
                            println!("    • {fund_type}: {url}");
                        }
                    }
                }
            }
        }
    }

    if !has_funding {
        print_info("No funding information found in installed packages");
    } else {
        println!("\n💙 Consider supporting these packages!");
    }

    Ok(())
}
