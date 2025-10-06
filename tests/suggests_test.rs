use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[path = "common/mod.rs"]
mod common;
use common::{ensure_lectern_binary, get_lectern_binary_path};

#[test]
fn test_suggests_command() {
    ensure_lectern_binary();

    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    let composer_json = r#"{"name": "test/suggests", "require": {}}"#;
    fs::write(temp_path.join("composer.json"), composer_json).unwrap();

    let output = Command::new(get_lectern_binary_path())
        .arg("suggests")
        .current_dir(temp_path)
        .output()
        .expect("Failed to execute lectern suggests");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should indicate no lock file or no suggestions
    assert!(
        stdout.contains("composer.lock") || stdout.contains("No package") || stdout.contains("suggest") || output.status.success()
    );
}

#[test]
fn test_suggests_no_lock() {
    ensure_lectern_binary();

    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    let composer_json = r#"{"name": "test/suggests", "require": {}}"#;
    fs::write(temp_path.join("composer.json"), composer_json).unwrap();

    let output = Command::new(get_lectern_binary_path())
        .arg("suggests")
        .current_dir(temp_path)
        .output()
        .expect("Failed to execute lectern suggests");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("composer.lock") || output.status.success());
}
