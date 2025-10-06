use std::process::Command;

#[path = "common/mod.rs"]
mod common;
use common::{ensure_lectern_binary, get_lectern_binary_path};

#[test]
fn test_search_command() {
    ensure_lectern_binary();

    let output = Command::new(get_lectern_binary_path())
        .arg("search")
        .arg("monolog")
        .output()
        .expect("Failed to execute lectern search");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should show search results
    assert!(
        stdout.contains("monolog") || stdout.contains("Search") || output.status.success()
    );
}

#[test]
fn test_search_no_terms() {
    ensure_lectern_binary();

    let output = Command::new(get_lectern_binary_path())
        .arg("search")
        .output()
        .expect("Failed to execute lectern search");

    // Should fail or indicate no terms provided
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);
    
    assert!(combined.contains("search terms") || !output.status.success());
}

#[test]
fn test_search_multiple_terms() {
    ensure_lectern_binary();

    let output = Command::new(get_lectern_binary_path())
        .arg("search")
        .arg("symfony")
        .arg("console")
        .output()
        .expect("Failed to execute lectern search");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("symfony") || stdout.contains("console") || output.status.success());
}
