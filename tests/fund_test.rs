use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[path = "common/mod.rs"]
mod common;
use common::{ensure_lectern_binary, get_lectern_binary_path};

#[test]
fn test_fund_command() {
    ensure_lectern_binary();

    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    let composer_json = r#"{"name": "test/fund", "require": {}}"#;
    fs::write(temp_path.join("composer.json"), composer_json).unwrap();

    let output = Command::new(get_lectern_binary_path())
        .arg("fund")
        .current_dir(temp_path)
        .output()
        .expect("Failed to execute lectern fund");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should indicate no lock file or no funding info
    assert!(
        stdout.contains("composer.lock") || stdout.contains("funding") || stdout.contains("No funding") || output.status.success()
    );
}

#[test]
fn test_fund_no_lock() {
    ensure_lectern_binary();

    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    let composer_json = r#"{"name": "test/fund", "require": {}}"#;
    fs::write(temp_path.join("composer.json"), composer_json).unwrap();

    let output = Command::new(get_lectern_binary_path())
        .arg("fund")
        .current_dir(temp_path)
        .output()
        .expect("Failed to execute lectern fund");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("composer.lock") || output.status.success());
}
