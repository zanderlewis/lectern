use crate::cli::ProhibitsArgs;
use crate::io::read_lock;
use crate::utils::{print_error, print_info, print_step};
use anyhow::Result;
use std::path::Path;

/// Show which packages prevent installing a given package
pub async fn show_prohibits(args: &ProhibitsArgs, working_dir: &Path) -> Result<()> {
    print_step(&format!(
        "🔍 Finding packages that conflict with {}...",
        args.package
    ));

    let lock_path = working_dir.join("composer.lock");
    if !lock_path.exists() {
        print_error("❌ No composer.lock found. Run 'lectern install' first.");
        return Ok(());
    }

    let lock = read_lock(&lock_path)?;
    let mut conflicts = Vec::new();

    // Check all packages for conflicts
    for pkg in lock.packages.iter().chain(lock.packages_dev.iter()) {
        if let Some(conflicts_map) = &pkg.conflict {
            if conflicts_map.contains_key(&args.package) {
                let constraint = conflicts_map.get(&args.package).unwrap();
                conflicts.push((pkg.name.clone(), constraint.clone()));
            }
        }
    }

    if conflicts.is_empty() {
        print_info(&format!("No packages conflict with {}", args.package));
    } else {
        println!("\n⚠️  Packages conflicting with {}:", args.package);
        for (name, constraint) in conflicts {
            println!("  • {} (conflicts with {})", name, constraint);
        }
    }

    Ok(())
}
