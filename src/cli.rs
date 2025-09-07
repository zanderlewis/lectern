use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "lectern",
    about = "Modern PHP dependency manager with Lectern.toml configuration",
    version
)]
pub struct Cli {
    /// Working directory (defaults to current dir)
    #[arg(long, short = 'd', default_value = ".")]
    pub working_dir: PathBuf,

    /// Verbosity level (-v, -vv, -vvv)
    #[arg(short = 'v', action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Suppress output
    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,

    /// Disable interaction
    #[arg(short = 'n', long = "no-interaction")]
    pub no_interaction: bool,

    /// Prefer stable packages
    #[arg(long = "prefer-stable")]
    pub prefer_stable: bool,

    /// Minimum stability level
    #[arg(long = "minimum-stability", default_value = "stable")]
    pub minimum_stability: String,

    /// Memory limit in MB
    #[arg(long = "memory-limit", default_value = "512")]
    pub memory_limit: u32,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Install packages from composer.json
    Install(InstallArgs),
    /// Update dependencies to latest versions
    Update(UpdateArgs),
    /// Add new packages to composer.json
    Require(RequireArgs),
    /// Remove packages from composer.json
    Remove(RemoveArgs),
    /// Show package information
    Show(ShowArgs),
    /// Show autoloader setup
    Autoload(DumpAutoloadArgs),
    /// Search for packages
    Search(SearchArgs),
    /// Initialize a new project
    Init(InitArgs),
    /// List outdated packages
    Outdated,
    /// List installed packages
    Status,
    /// Show licenses of dependencies
    Licenses,
    /// Validate composer.json
    Validate(ValidateArgs),
}

#[derive(Args, Debug)]
pub struct InstallArgs {
    /// Don't install dev dependencies
    #[arg(long = "no-dev")]
    pub no_dev: bool,

    /// Dry run mode
    #[arg(long = "dry-run")]
    pub dry_run: bool,

    /// Prefer source installs
    #[arg(long = "prefer-source")]
    pub prefer_source: bool,

    /// Prefer dist installs
    #[arg(long = "prefer-dist")]
    pub prefer_dist: bool,

    /// Ignore platform requirements
    #[arg(long = "ignore-platform-reqs")]
    pub ignore_platform_reqs: bool,

    /// Optimize autoloader
    #[arg(long = "optimize-autoloader")]
    pub optimize_autoloader: bool,
}

#[derive(Args, Debug)]
pub struct UpdateArgs {
    /// Packages to update (empty = all)
    pub packages: Vec<String>,

    /// Don't update dev dependencies
    #[arg(long = "no-dev")]
    pub no_dev: bool,

    /// Dry run mode
    #[arg(long = "dry-run")]
    pub dry_run: bool,

    /// Update to latest versions (ignoring constraints)
    #[arg(long = "with-all-dependencies")]
    pub with_all_dependencies: bool,

    /// Prefer source installs
    #[arg(long = "prefer-source")]
    pub prefer_source: bool,

    /// Prefer dist installs
    #[arg(long = "prefer-dist")]
    pub prefer_dist: bool,

    /// Ignore platform requirements
    #[arg(long = "ignore-platform-reqs")]
    pub ignore_platform_reqs: bool,

    /// Optimize autoloader
    #[arg(long = "optimize-autoloader")]
    pub optimize_autoloader: bool,
}

#[derive(Args, Debug)]
pub struct RequireArgs {
    /// Packages to add (format: vendor/package:constraint)
    pub packages: Vec<String>,

    /// Add to dev dependencies
    #[arg(long = "dev")]
    pub dev: bool,

    /// Dry run mode
    #[arg(long = "dry-run")]
    pub dry_run: bool,

    /// Don't update dependencies after require
    #[arg(long = "no-update")]
    pub no_update: bool,

    /// Update with dependencies
    #[arg(long = "update-with-dependencies")]
    pub update_with_dependencies: bool,

    /// Ignore platform requirements
    #[arg(long = "ignore-platform-reqs")]
    pub ignore_platform_reqs: bool,
}

#[derive(Args, Debug)]
pub struct RemoveArgs {
    /// Packages to remove
    pub packages: Vec<String>,

    /// Remove from dev dependencies
    #[arg(long = "dev")]
    pub dev: bool,

    /// Dry run mode
    #[arg(long = "dry-run")]
    pub dry_run: bool,

    /// Don't update after removal
    #[arg(long = "no-update")]
    pub no_update: bool,

    /// Update with dependencies
    #[arg(long = "update-with-dependencies")]
    pub update_with_dependencies: bool,
}

#[derive(Args, Debug)]
pub struct ShowArgs {
    /// Package name to show info for
    pub package: Option<String>,

    /// Show available versions
    #[arg(long = "available")]
    pub available: bool,

    /// Include platform packages
    #[arg(long = "platform")]
    pub platform: bool,

    /// Show only direct dependencies
    #[arg(long = "direct")]
    pub direct: bool,

    /// Show dependency tree
    #[arg(long = "tree")]
    pub tree: bool,

    /// Output format (table, json)
    #[arg(long = "format", default_value = "table")]
    pub format: String,
}

#[derive(Args, Debug)]
pub struct DumpAutoloadArgs {
    /// Optimize autoloader (PSR-4 to classmap)
    #[arg(long = "optimize", short = 'o')]
    pub optimize: bool,

    /// Generate authoritative classmap
    #[arg(long = "classmap-authoritative")]
    pub classmap_authoritative: bool,

    /// Use APCu cache
    #[arg(long = "apcu")]
    pub apcu: bool,

    /// Don't include dev autoload
    #[arg(long = "no-dev")]
    pub no_dev: bool,
}

#[derive(Args, Debug)]
pub struct SearchArgs {
    /// Search terms
    pub terms: Vec<String>,
}

#[derive(Args, Debug)]
pub struct InitArgs {
    /// Project name
    #[arg(long = "name")]
    pub name: Option<String>,

    /// Project description
    #[arg(long = "description")]
    pub description: Option<String>,

    /// Author name
    #[arg(long = "author")]
    pub author: Option<String>,

    /// Project type
    #[arg(long = "type", default_value = "library")]
    pub project_type: String,

    /// Homepage URL
    #[arg(long = "homepage")]
    pub homepage: Option<String>,

    /// Require dependencies interactively
    #[arg(long = "require")]
    pub require: bool,

    /// Require dev dependencies interactively
    #[arg(long = "require-dev")]
    pub require_dev: bool,

    /// Minimum stability
    #[arg(long = "stability")]
    pub stability: Option<String>,

    /// License
    #[arg(long = "license")]
    pub license: Option<String>,

    /// Repository type
    #[arg(long = "repository")]
    pub repository: Option<String>,
}

#[derive(Args, Debug)]
pub struct ValidateArgs {
    /// Don't check for publish issues
    #[arg(long = "no-check-publish")]
    pub no_check_publish: bool,

    /// Check for typos in composer.json
    #[arg(long = "with-dependencies")]
    pub with_dependencies: bool,

    /// Strict validation
    #[arg(long = "strict")]
    pub strict: bool,
}
