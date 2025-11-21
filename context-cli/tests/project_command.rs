use anyhow::Result;
use assert_cmd::Command;
use context_core::Document;
use tempfile::tempdir;

#[test]
fn current_defaults_to_default_project() -> Result<()> {
    let temp = tempdir()?;
    let home = temp.path().join("home");
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("context-cli"));
    let assert = cmd
        .env("CONTEXT_HOME", &home)
        .current_dir(temp.path())
        .args(["--json", "project", "current"])
        .assert()
        .success();

    let stdout = assert.get_output().stdout.clone();
    let payload: serde_json::Value = serde_json::from_slice(&stdout)?;
    assert_eq!(payload["project"], "default");

    Ok(())
}

#[test]
fn set_updates_current_and_is_used_by_other_commands() -> Result<()> {
    let temp = tempdir()?;
    let home = temp.path().join("home");

    let mut set_cmd = Command::new(assert_cmd::cargo::cargo_bin!("context-cli"));
    set_cmd
        .env("CONTEXT_HOME", &home)
        .current_dir(temp.path())
        .args(["--json", "project", "set", "demo-project"])
        .assert()
        .success();

    let mut current_cmd = Command::new(assert_cmd::cargo::cargo_bin!("context-cli"));
    let current = current_cmd
        .env("CONTEXT_HOME", &home)
        .current_dir(temp.path())
        .args(["--json", "project", "current"])
        .assert()
        .success();
    let payload: serde_json::Value = serde_json::from_slice(&current.get_output().stdout)?;
    assert_eq!(payload["project"], "demo-project");

    let mut ls_cmd = Command::new(assert_cmd::cargo::cargo_bin!("context-cli"));
    let ls = ls_cmd
        .env("CONTEXT_HOME", &home)
        .current_dir(temp.path())
        .args(["--json", "ls"])
        .assert()
        .success();
    let docs: Vec<Document> = serde_json::from_slice(&ls.get_output().stdout)?;
    assert!(docs.iter().all(|doc| doc.project == "demo-project"));

    Ok(())
}

#[test]
fn list_returns_known_projects() -> Result<()> {
    let temp = tempdir()?;
    let home = temp.path().join("home");

    let mut set_cmd = Command::new(assert_cmd::cargo::cargo_bin!("context-cli"));
    set_cmd
        .env("CONTEXT_HOME", &home)
        .current_dir(temp.path())
        .args(["--json", "project", "set", "alpha"])
        .assert()
        .success();

    let mut set_cmd2 = Command::new(assert_cmd::cargo::cargo_bin!("context-cli"));
    set_cmd2
        .env("CONTEXT_HOME", &home)
        .current_dir(temp.path())
        .args(["--json", "project", "set", "bravo"])
        .assert()
        .success();

    let mut list_cmd = Command::new(assert_cmd::cargo::cargo_bin!("context-cli"));
    let list = list_cmd
        .env("CONTEXT_HOME", &home)
        .current_dir(temp.path())
        .args(["--json", "project", "list"])
        .assert()
        .success();

    let projects: Vec<String> = serde_json::from_slice(&list.get_output().stdout)?;
    assert!(projects.contains(&"alpha".to_string()));
    assert!(projects.contains(&"bravo".to_string()));
    assert!(projects.contains(&"default".to_string()));

    Ok(())
}
