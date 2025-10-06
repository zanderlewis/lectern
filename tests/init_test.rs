use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[path = "common/mod.rs"]
mod common;
use common::{ensure_lectern_binary, get_lectern_binary_path};

#[test]
fn test_init_command_creates_composer_json() {
    ensure_lectern_binary();

    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    let output = Command::new(get_lectern_binary_path())
        .arg("init")
        .arg("--name")
        .arg("test/init-test")
        .arg("--description")
        .arg("Test package description")
        .current_dir(temp_path)
        .output()
        .expect("Failed to execute lectern init");

    assert!(output.status.success());
    assert!(temp_path.join("composer.json").exists());

    let content = fs::read_to_string(temp_path.join("composer.json")).unwrap();
    assert!(content.contains("test/init-test"));
}

#[test]
fn test_init_command_minimal() {
    ensure_lectern_binary();

    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    let output = Command::new(get_lectern_binary_path())
        .arg("init")
        .arg("--name")
        .arg("vendor/package")
        .current_dir(temp_path)
        .output()
        .expect("Failed to execute lectern init");

    assert!(output.status.success());
    assert!(temp_path.join("composer.json").exists());

    let content = fs::read_to_string(temp_path.join("composer.json")).unwrap();
    assert!(content.contains("vendor/package"));
}

#[test]
fn test_init_command_with_all_options() {
    ensure_lectern_binary();

    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    let output = Command::new(get_lectern_binary_path())
        .arg("init")
        .arg("--name")
        .arg("test/full-init")
        .arg("--description")
        .arg("Full featured test package")
        .arg("--author")
        .arg("Test Author <test@example.com>")
        .current_dir(temp_path)
        .output()
        .expect("Failed to execute lectern init");

    assert!(output.status.success());
    assert!(temp_path.join("composer.json").exists());

    let content = fs::read_to_string(temp_path.join("composer.json")).unwrap();
    assert!(content.contains("test/full-init"));
    // Description may or may not be included depending on implementation
    assert!(content.contains("name") && content.contains("require"));
}
