use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[path = "common/mod.rs"]
mod common;
use common::{ensure_lectern_binary, get_lectern_binary_path};

#[test]
fn test_prohibits_command() {
    ensure_lectern_binary();

    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    let composer_json = r#"{"name": "test/prohibits", "require": {}}"#;
    fs::write(temp_path.join("composer.json"), composer_json).unwrap();

    let output = Command::new(get_lectern_binary_path())
        .arg("prohibits")
        .arg("some/package")
        .current_dir(temp_path)
        .output()
        .expect("Failed to execute lectern prohibits");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("composer.lock") || stdout.contains("No packages") || output.status.success(),
        "Should handle missing lock file"
    );
}

#[test]
fn test_prohibits_no_conflicts() {
    ensure_lectern_binary();

    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    let composer_json = r#"{"name": "test/prohibits", "require": {}}"#;
    fs::write(temp_path.join("composer.json"), composer_json).unwrap();
    
    let lock_json = r#"{"packages":[],"packages-dev":[]}"#;
    fs::write(temp_path.join("composer.lock"), lock_json).unwrap();

    let output = Command::new(get_lectern_binary_path())
        .arg("prohibits")
        .arg("some/package")
        .current_dir(temp_path)
        .output()
        .expect("Failed to execute lectern prohibits");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No packages") || stdout.contains("conflict") || output.status.success());
}
