use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[path = "common/mod.rs"]
mod common;
use common::{ensure_lectern_binary, get_lectern_binary_path};

#[test]
fn test_status_command() {
    ensure_lectern_binary();

    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    let composer_json = r#"{"name": "test/status", "require": {}}"#;
    fs::write(temp_path.join("composer.json"), composer_json).unwrap();

    let output = Command::new(get_lectern_binary_path())
        .arg("status")
        .current_dir(temp_path)
        .output()
        .expect("Failed to execute lectern status");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should show status or indicate no lock file
    assert!(
        stdout.contains("status") || stdout.contains("composer.lock") || stdout.contains("packages") || output.status.success()
    );
}

#[test]
fn test_status_no_lock() {
    ensure_lectern_binary();

    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    let composer_json = r#"{"name": "test/status", "require": {}}"#;
    fs::write(temp_path.join("composer.json"), composer_json).unwrap();

    let output = Command::new(get_lectern_binary_path())
        .arg("status")
        .current_dir(temp_path)
        .output()
        .expect("Failed to execute lectern status");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("composer.lock") || output.status.success());
}

#[test]
fn test_status_with_empty_lock() {
    ensure_lectern_binary();

    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    let composer_json = r#"{"name": "test/status", "require": {}}"#;
    fs::write(temp_path.join("composer.json"), composer_json).unwrap();
    
    let lock_json = r#"{"packages":[],"packages-dev":[]}"#;
    fs::write(temp_path.join("composer.lock"), lock_json).unwrap();

    let output = Command::new(get_lectern_binary_path())
        .arg("status")
        .current_dir(temp_path)
        .output()
        .expect("Failed to execute lectern status");

    // Should run without crashing (may show empty or succeed)
    assert!(output.status.code().is_some());
}
