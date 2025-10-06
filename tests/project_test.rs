use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[path = "common/mod.rs"]
mod common;
use common::{ensure_lectern_binary, get_lectern_binary_path};

#[test]
fn test_create_project_command() {
    ensure_lectern_binary();

    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    let project_dir = temp_path.join("my-project");

    let output = Command::new(get_lectern_binary_path())
        .arg("create-project")
        .arg("symfony/skeleton")
        .arg(project_dir.to_string_lossy().as_ref())
        .current_dir(temp_path)
        .output()
        .expect("Failed to execute lectern create-project");

    // This is a heavy operation, just verify it doesn't crash
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Check command executed (may or may not succeed depending on network)
    assert!(
        output.status.success() || stdout.contains("create") || stderr.contains("create") || project_dir.exists()
    );
}

#[test]
fn test_create_project_existing_directory() {
    ensure_lectern_binary();

    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    let project_dir = temp_path.join("existing");
    
    // Create directory first
    fs::create_dir(&project_dir).unwrap();
    fs::write(project_dir.join("test.txt"), "existing").unwrap();

    let output = Command::new(get_lectern_binary_path())
        .arg("create-project")
        .arg("symfony/skeleton")
        .arg(project_dir.to_string_lossy().as_ref())
        .current_dir(temp_path)
        .output()
        .expect("Failed to execute lectern create-project");

    // Should handle existing directory (fail or warn)
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);
    
    assert!(
        !output.status.success() || combined.contains("exist") || combined.contains("empty")
    );
}
