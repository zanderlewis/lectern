use std::process::Command;

#[path = "common/mod.rs"]
mod common;
use common::{ensure_lectern_binary, get_lectern_binary_path};

#[test]
fn test_clear_cache_command() {
    ensure_lectern_binary();

    let output = Command::new(get_lectern_binary_path())
        .arg("clear-cache")
        .arg("all")
        .output()
        .expect("Failed to execute lectern clear-cache");

    // Should succeed or report no cache found
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success() || stdout.contains("No cache"),
        "Clear cache should succeed or report no cache"
    );
}

#[test]
fn test_clear_cache_repo_only() {
    ensure_lectern_binary();

    let output = Command::new(get_lectern_binary_path())
        .arg("clear-cache")
        .arg("repo")
        .output()
        .expect("Failed to execute lectern clear-cache repo");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success() || stdout.contains("No cache") || stdout.contains("cleared")
    );
}

#[test]
fn test_clear_cache_files_only() {
    ensure_lectern_binary();

    let output = Command::new(get_lectern_binary_path())
        .arg("clear-cache")
        .arg("files")
        .output()
        .expect("Failed to execute lectern clear-cache files");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success() || stdout.contains("No cache") || stdout.contains("cleared")
    );
}

#[test]
fn test_clear_cache_invalid_type() {
    ensure_lectern_binary();

    let output = Command::new(get_lectern_binary_path())
        .arg("clear-cache")
        .arg("invalid")
        .output()
        .expect("Failed to execute lectern clear-cache");

    // Should either fail or report unknown type
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);
    
    assert!(
        !output.status.success() || combined.contains("Unknown") || combined.contains("invalid")
    );
}
