use crate::cli::DependsArgs;
use crate::io::read_lock;
use crate::utils::{print_error, print_info, print_step};
use anyhow::Result;
use std::path::Path;

/// Show which packages depend on a given package
pub async fn show_depends(args: &DependsArgs, working_dir: &Path) -> Result<()> {
    print_step(&format!(
        "🔍 Finding packages that depend on {}...",
        args.package
    ));

    let lock_path = working_dir.join("composer.lock");
    if !lock_path.exists() {
        print_error("❌ No composer.lock found. Run 'lectern install' first.");
        return Ok(());
    }

    let lock = read_lock(&lock_path)?;
    let mut dependents = Vec::new();

    // Check all packages
    for pkg in lock.packages.iter().chain(lock.packages_dev.iter()) {
        if let Some(requires) = &pkg.require {
            if requires.contains_key(&args.package) {
                let constraint = requires.get(&args.package).unwrap();
                dependents.push((pkg.name.clone(), constraint.clone(), false));
            }
        }
    }

    if dependents.is_empty() {
        print_info(&format!("No packages depend on {}", args.package));
    } else {
        println!("\n📦 Packages depending on {}:", args.package);
        for (name, constraint, _) in dependents {
            println!("  • {} (requires {})", name, constraint);
        }
    }

    Ok(())
}
