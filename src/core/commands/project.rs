use crate::cli::CreateProjectArgs;
use crate::utils::{print_info, print_step, print_success};
use anyhow::{Result, anyhow};
use std::collections::BTreeMap;
use std::path::Path;

/// Create a new project from a package
pub async fn create_project(args: &CreateProjectArgs, working_dir: &Path) -> Result<()> {
    print_step(&format!("📦 Creating new project from {}...", args.package));

    let target_dir = if let Some(dir) = &args.directory {
        working_dir.join(dir)
    } else {
        let pkg_name = args.package.split('/').last().unwrap_or(&args.package);
        working_dir.join(pkg_name)
    };

    if target_dir.exists() {
        return Err(anyhow!(
            "Target directory {} already exists",
            target_dir.display()
        ));
    }

    std::fs::create_dir_all(&target_dir)?;

    print_info(&format!(
        "📥 Fetching package information for {}...",
        args.package
    ));

    // For now, just create a basic composer.json with the package as a dependency
    // A full implementation would download and extract the package's skeleton
    let composer = crate::models::model::ComposerJson {
        name: Some(args.package.clone()),
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
        require: [(
            args.package.clone(),
            args.version.clone().unwrap_or_else(|| "*".to_string()),
        )]
        .iter()
        .cloned()
        .collect(),
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
        minimum_stability: None,
        prefer_stable: Some(true),
        bin: None,
    };

    let composer_json = serde_json::to_string_pretty(&composer)?;
    std::fs::write(target_dir.join("composer.json"), composer_json)?;

    print_success("✅ Project created successfully");
    print_info(&format!(
        "Run 'cd {}' and 'lectern install' to set up dependencies",
        target_dir.file_name().unwrap().to_string_lossy()
    ));

    Ok(())
}
