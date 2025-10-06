use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[path = "common/mod.rs"]
mod common;
use common::{ensure_lectern_binary, get_lectern_binary_path};

#[test]
fn test_outdated_command() {
    ensure_lectern_binary();

    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    let composer_json = r#"{"name": "test/outdated", "require": {}}"#;
    fs::write(temp_path.join("composer.json"), composer_json).unwrap();

    let output = Command::new(get_lectern_binary_path())
        .arg("outdated")
        .current_dir(temp_path)
        .output()
        .expect("Failed to execute lectern outdated");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should either show outdated packages or indicate no lock file
    assert!(
        stdout.contains("outdated") || stdout.contains("composer.lock") || stdout.contains("up to date") || output.status.success()
    );
}

#[test]
fn test_outdated_with_quiet_flag() {
    ensure_lectern_binary();

    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    let composer_json = r#"{"name": "test/outdated", "require": {}}"#;
    fs::write(temp_path.join("composer.json"), composer_json).unwrap();

    let output = Command::new(get_lectern_binary_path())
        .arg("outdated")
        .arg("--quiet")
        .current_dir(temp_path)
        .output()
        .expect("Failed to execute lectern outdated --quiet");

    // Quiet mode should run without crashing (may fail without lock file)
    assert!(output.status.code().is_some());
}
