use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// Executes a git command on a path without flashing console windows.
pub fn run_git<P: AsRef<Path>>(dir: P, args: &[&str]) -> Result<String, String> {
    let mut cmd = Command::new("git");
    cmd.current_dir(dir.as_ref()).args(args);
    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);

    let output = cmd.output().map_err(|e| format!("Failed to spawn git: {}", e))?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(err.trim().to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Initializes a real temporary git repository with an initial commit on 'main'.
pub fn create_test_repo() -> TempDir {
    let temp = tempfile::tempdir().expect("failed to create temp dir");
    let p = temp.path();

    run_git(p, &["init", "-b", "main"]).expect("git init failed");
    run_git(p, &["config", "user.name", "Integration Tester"]).expect("config user.name failed");
    run_git(p, &["config", "user.email", "tester@workspace-companion.test"])
        .expect("config user.email failed");

    let readme = p.join("README.md");
    fs::write(&readme, "# Integration Test Workspace\n").expect("write readme failed");
    run_git(p, &["add", "README.md"]).expect("git add failed");
    run_git(p, &["commit", "-m", "Initial commit on main"]).expect("git commit failed");

    temp
}
