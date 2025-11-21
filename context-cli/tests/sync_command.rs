use anyhow::Result;
use assert_cmd::Command;

#[test]
fn sync_command_is_not_exposed() -> Result<()> {
    let assert = Command::new(assert_cmd::cargo::cargo_bin!("context-cli"))
        .args(["sync", "status"])
        .assert()
        .failure();

    let stderr = String::from_utf8_lossy(&assert.get_output().stderr).to_lowercase();
    assert!(
        stderr.contains("subcommand") || stderr.contains("found argument"),
        "expected clap to report missing subcommand; stderr: {stderr}"
    );
    assert!(
        stderr.contains("sync"),
        "stderr should mention sync command; stderr: {stderr}"
    );

    Ok(())
}
