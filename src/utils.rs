use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;

/// Normalize a repo path: absolute if relative
pub fn normalize_path(s: &str) -> Result<PathBuf> {
    let p = PathBuf::from(s);
    if p.is_absolute() {
        return Ok(p);
    }
    let cur = std::env::current_dir()?;
    Ok(cur.join(p))
}

/// Clean first path component from archive entries
pub fn strip_first_component(s: &str) -> std::path::PathBuf {
    let p = std::path::Path::new(s);
    let mut comps = p.components();
    comps.next();
    comps.as_path().to_path_buf()
}

/// Print a success message in green
pub fn print_success(message: &str) {
    println!("{} {}", "[SUCCESS]".green().bold(), message);
}

/// Print an info message in blue
pub fn print_info(message: &str) {
    println!("{} {}", "[INFO]".blue().bold(), message);
}

/// Print an error message in red
pub fn print_error(message: &str) {
    eprintln!("{} {}", "[ERROR]".red().bold(), message);
}

/// Print a warning message in yellow
pub fn print_warning(message: &str) {
    println!("{} {}", "[WARNING]".yellow().bold(), message);
}

/// Print a step message (for showing progress)
pub fn print_step(message: &str) {
    println!("{} {}", "[STEP]".cyan().bold(), message);
}
