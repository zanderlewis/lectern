use anyhow::Result;
use clap::Parser;
use env_logger::Env;
use lectern::{
    autoload::write_autoload_files,
    cli::*,
    commands::{
        browse_package, check_outdated_packages, clear_cache, create_project, diagnose, run_script,
        search_packages, show_dependency_licenses, show_dependency_status, show_depends,
        show_funding, show_package_details, show_prohibits, show_suggests,
    },
    installer::{InstalledPackage, install_packages},
    io::{read_composer_json, read_lock, write_lock},
    models::model::*,
    resolver::solve,
    utils::*,
};
use std::collections::BTreeMap;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // Parse CLI arguments
    let cli = Cli::parse();

    // Set working directory
    let working_dir = &cli.working_dir;

    // Execute the requested command
    match cli.command {
        Some(command) => match command {
            Commands::Install(args) => {
                if args.dry_run {
                    print_info("🔍 Dry run mode - no changes will be made");
                }

                let composer_path = working_dir.join("composer.json");
                let composer = read_composer_json(&composer_path)?;

                if !args.dry_run {
                    let lock = solve(&composer).await?;
                    let lock_path = working_dir.join("composer.lock");
                    write_lock(&lock_path, &lock)?;
                    install_packages(&lock.packages, working_dir).await?;
                } else {
                    print_success("✅ Dry run completed - dependencies would be installed");
                }
            }

            Commands::Update(args) => {
                if args.dry_run {
                    print_info("🔍 Dry run mode - no changes will be made");
                }

                let composer_path = working_dir.join("composer.json");
                let composer = read_composer_json(&composer_path)?;

                if !args.dry_run {
                    let lock = solve(&composer).await?;
                    let lock_path = working_dir.join("composer.lock");
                    write_lock(&lock_path, &lock)?;
                    install_packages(&lock.packages, working_dir).await?;
                } else {
                    print_success("✅ Dry run completed - dependencies would be updated");
                }
            }

            Commands::Require(args) => {
                if args.dry_run {
                    print_info("🔍 Dry run mode - no changes will be made");
                }

                let composer_path = working_dir.join("composer.json");
                let mut composer = read_composer_json(&composer_path)?;

                // Add packages to composer.json
                for package_spec in &args.packages {
                    let (name, constraint) = if let Some(pos) = package_spec.find(':') {
                        (
                            package_spec[..pos].to_string(),
                            package_spec[pos + 1..].to_string(),
                        )
                    } else {
                        (package_spec.clone(), "*".to_string())
                    };

                    if args.dev {
                        composer.require_dev.insert(name, constraint);
                    } else {
                        composer.require.insert(name, constraint);
                    }
                }

                if !args.dry_run {
                    // Write updated composer.json
                    let composer_json = serde_json::to_string_pretty(&composer)?;
                    std::fs::write(&composer_path, composer_json)?;

                    if !args.no_update {
                        let lock = solve(&composer).await?;
                        let lock_path = working_dir.join("composer.lock");
                        write_lock(&lock_path, &lock)?;
                        install_packages(&lock.packages, working_dir).await?;
                    }
                } else {
                    print_success("✅ Dry run completed - packages would be added");
                }
            }

            Commands::Remove(args) => {
                if args.dry_run {
                    print_info("🔍 Dry run mode - no changes will be made");
                }

                let composer_path = working_dir.join("composer.json");
                let mut composer = read_composer_json(&composer_path)?;

                // Remove packages from composer.json
                for package_name in &args.packages {
                    if args.dev {
                        composer.require_dev.remove(package_name);
                    } else {
                        composer.require.remove(package_name);
                    }
                }

                if !args.dry_run {
                    // Write updated composer.json
                    let composer_json = serde_json::to_string_pretty(&composer)?;
                    std::fs::write(&composer_path, composer_json)?;

                    if !args.no_update {
                        let lock = solve(&composer).await?;
                        let lock_path = working_dir.join("composer.lock");
                        write_lock(&lock_path, &lock)?;
                        install_packages(&lock.packages, working_dir).await?;
                    }
                } else {
                    print_success("✅ Dry run completed - packages would be removed");
                }
            }

            Commands::Show(args) => {
                if let Some(package) = &args.package {
                    show_package_details(package, working_dir).await?;
                } else {
                    show_dependency_status(working_dir).await?;
                }
            }

            Commands::Autoload(_args) => {
                let composer_path = working_dir.join("composer.json");
                let composer = read_composer_json(&composer_path)?;

                // Read the lock file to get installed packages
                let lock_path = working_dir.join("composer.lock");
                if !lock_path.exists() {
                    print_error("❌ No composer.lock found. Run 'lectern install' first.");
                    return Ok(());
                }

                let lock = read_lock(&lock_path)?;

                // Convert LockedPackage to InstalledPackage for autoload generation
                let installed: Vec<InstalledPackage> = lock
                    .packages
                    .iter()
                    .map(|pkg| InstalledPackage {
                        name: pkg.name.clone(),
                        version: pkg.version.clone(),
                        path: format!("vendor/{}", pkg.name).into(),
                    })
                    .collect();

                write_autoload_files(working_dir, &composer, &installed).await?;
            }

            Commands::Search(args) => {
                search_packages(&args.terms, working_dir).await?;
            }

            Commands::Init(args) => {
                init_project(working_dir, &args)?;
            }

            Commands::Outdated => {
                check_outdated_packages(working_dir, cli.quiet).await?;
            }

            Commands::Status => {
                show_dependency_status(working_dir).await?;
            }

            Commands::Licenses => {
                show_dependency_licenses(working_dir, cli.quiet).await?;
            }

            Commands::Validate(args) => {
                validate_composer_json(working_dir, &args)?;
            }

            Commands::CreateProject(args) => {
                create_project(&args, working_dir).await?;
            }

            Commands::DumpAutoload(_) => {
                let composer_path = working_dir.join("composer.json");
                let composer = read_composer_json(&composer_path)?;
                let lock_path = working_dir.join("composer.lock");

                if !lock_path.exists() {
                    print_error("❌ No composer.lock found. Run 'lectern install' first.");
                    return Ok(());
                }

                let lock = read_lock(&lock_path)?;
                let installed: Vec<InstalledPackage> = lock
                    .packages
                    .iter()
                    .map(|pkg| InstalledPackage {
                        name: pkg.name.clone(),
                        version: pkg.version.clone(),
                        path: format!("vendor/{}", pkg.name).into(),
                    })
                    .collect();

                write_autoload_files(working_dir, &composer, &installed).await?;
                print_success("✅ Generated autoload files");
            }

            Commands::RunScript(args) => {
                run_script(&args, working_dir).await?;
            }

            Commands::Diagnose => {
                diagnose(working_dir).await?;
            }

            Commands::Archive(_args) => {
                print_info("📦 Archive command not yet fully implemented");
                // TODO: Implement archive functionality
            }

            Commands::ClearCache(args) => {
                clear_cache(&args).await?;
            }

            Commands::Config(_args) => {
                print_info("⚙️  Config command not yet fully implemented");
                // TODO: Implement config management
            }

            Commands::Depends(args) => {
                show_depends(&args, working_dir).await?;
            }

            Commands::Prohibits(args) => {
                show_prohibits(&args, working_dir).await?;
            }

            Commands::Browse(args) => {
                browse_package(&args).await?;
            }

            Commands::Suggests => {
                show_suggests(working_dir).await?;
            }

            Commands::Fund => {
                show_funding(working_dir).await?;
            }
        },
        _ => {
            // No command provided, show help
            use clap::CommandFactory;
            Cli::command().print_help()?;
        }
    }

    Ok(())
}

/// Initialize a new project
fn init_project(working_dir: &std::path::Path, args: &InitArgs) -> Result<()> {
    print_step("📝 Initializing new project...");

    let composer_path = working_dir.join("composer.json");

    if composer_path.exists() {
        print_error("❌ composer.json already exists");
        return Ok(());
    }

    let composer = ComposerJson {
        name: args.name.clone(),
        description: None,
        version: None,
        package_type: None,
        keywords: None,
        homepage: None,
        readme: None,
        time: None,
        license: None,
        authors: None,
        support: None,
        require: BTreeMap::new(),
        require_dev: BTreeMap::new(),
        conflict: None,
        replace: None,
        provide: None,
        suggest: None,
        autoload: None,
        autoload_dev: None,
        include_path: None,
        target_dir: None,
        repositories: None,
        config: None,
        scripts: None,
        extra: None,
        minimum_stability: args.stability.clone(),
        prefer_stable: Some(true),
        bin: None,
    };

    // Interactive package requirements
    if args.require || args.require_dev {
        print_info("📦 Interactive package selection not yet implemented");
    }

    let composer_json = serde_json::to_string_pretty(&composer)?;
    std::fs::write(&composer_path, composer_json)?;

    print_success("✅ Created composer.json");
    Ok(())
}

/// Validate composer.json
fn validate_composer_json(working_dir: &std::path::Path, _args: &ValidateArgs) -> Result<()> {
    print_step("🔍 Validating composer.json...");

    let composer_path = working_dir.join("composer.json");

    if !composer_path.exists() {
        print_error("❌ composer.json not found");
        return Ok(());
    }

    match read_composer_json(&composer_path) {
        Ok(_) => {
            print_success("✅ composer.json is valid");
        }
        Err(e) => {
            print_error(&format!("❌ composer.json is invalid: {e}"));
        }
    }

    Ok(())
}
