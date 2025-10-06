use crate::io::{read_composer_json, read_lock};
use crate::utils::{print_info, print_step, print_success};
use anyhow::Result;
use std::path::Path;

/// Diagnose the system to identify common problems
pub async fn diagnose(working_dir: &Path) -> Result<()> {
    print_step("🔍 Running diagnostics...");

    let mut issues: Vec<String> = Vec::new();
    let mut checks_passed = 0;

    // Check composer.json
    print_info("Checking composer.json...");
    let composer_path = working_dir.join("composer.json");
    if !composer_path.exists() {
        issues.push("❌ composer.json not found".to_string());
    } else {
        match read_composer_json(&composer_path) {
            Ok(_) => {
                checks_passed += 1;
                println!("  ✓ composer.json is valid");
            }
            Err(e) => {
                issues.push(format!("❌ composer.json is invalid: {e}"));
            }
        }
    }

    // Check composer.lock
    print_info("Checking composer.lock...");
    let lock_path = working_dir.join("composer.lock");
    if !lock_path.exists() {
        issues.push("⚠️  composer.lock not found (run 'lectern install')".to_string());
    } else {
        match read_lock(&lock_path) {
            Ok(_) => {
                checks_passed += 1;
                println!("  ✓ composer.lock is valid");
            }
            Err(e) => {
                issues.push(format!("❌ composer.lock is invalid: {e}"));
            }
        }
    }

    // Check vendor directory
    print_info("Checking vendor directory...");
    let vendor_path = working_dir.join("vendor");
    if !vendor_path.exists() {
        issues.push("⚠️  vendor directory not found (run 'lectern install')".to_string());
    } else {
        checks_passed += 1;
        println!("  ✓ vendor directory exists");
    }

    // Check cache directory
    print_info("Checking cache directory...");
    let cache_dir = crate::core::cache_utils::get_cache_dir();
    if cache_dir.exists() {
        checks_passed += 1;
        println!("  ✓ Cache directory: {}", cache_dir.display());

        // Check cache size
        if let Ok(size) = get_dir_size(&cache_dir) {
            println!(
                "    Cache size: {:.2} MB",
                size as f64 / 1024.0 / 1024.0
            );
        }
    } else {
        println!("  ℹ️  Cache directory will be created on first use");
    }

    // Check PHP (if available)
    print_info("Checking PHP availability...");
    if let Ok(output) = std::process::Command::new("php")
        .arg("--version")
        .output()
    {
        if output.status.success() {
            checks_passed += 1;
            if let Ok(version) = String::from_utf8(output.stdout) {
                let first_line = version.lines().next().unwrap_or("Unknown");
                println!("  ✓ {first_line}");
            }
        }
    } else {
        issues.push("⚠️  PHP not found in PATH".to_string());
    }

    // Summary
    println!("\n📊 Diagnostic Summary:");
    println!("  Checks passed: {checks_passed}");
    println!("  Issues found: {}", issues.len());

    if !issues.is_empty() {
        println!("\n⚠️  Issues:");
        for issue in issues {
            println!("  {issue}");
        }
    } else {
        print_success("✅ No issues detected!");
    }

    Ok(())
}

/// Helper function to calculate directory size
fn get_dir_size(path: &Path) -> Result<u64> {
    let mut size = 0;
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            if metadata.is_dir() {
                size += get_dir_size(&entry.path())?;
            } else {
                size += metadata.len();
            }
        }
    }
    Ok(size)
}
