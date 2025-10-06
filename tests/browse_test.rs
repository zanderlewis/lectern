use std::process::Command;

#[path = "common/mod.rs"]
mod common;
use common::{ensure_lectern_binary, get_lectern_binary_path};

#[test]
fn test_browse_command() {
    ensure_lectern_binary();

    let output = Command::new(get_lectern_binary_path())
        .arg("browse")
        .arg("--show")  // Just print URL, don't open browser
        .arg("monolog/monolog")
        .output()
        .expect("Failed to execute lectern browse");

    // Should print the repository URL
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Should succeed and output a URL
    assert!(output.status.success());
    assert!(stdout.contains("http") || stdout.contains("github") || stdout.contains("gitlab"));
}

#[test]
fn test_browse_nonexistent() {
    ensure_lectern_binary();

    let output = Command::new(get_lectern_binary_path())
        .arg("browse")
        .arg("--show")  // Just print URL, don't open browser
        .arg("nonexistent/package")
        .output()
        .expect("Failed to execute lectern browse");

    // Should handle gracefully - might fail or print warning
    assert!(output.status.code().is_some());
}
