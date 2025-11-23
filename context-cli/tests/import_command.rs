use anyhow::Result;
use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use tempfile::tempdir;

#[test]
fn import_yes_json_imports_markdown_files() -> Result<()> {
    let dir = tempdir()?;
    let root = dir.path();

    fs::write(root.join("notes.md"), "# Notes\nbody")?;
    fs::create_dir_all(root.join("agents"))?;
    fs::write(root.join("agents/plan.md"), "# Plan\n- item")?;
    fs::write(root.join("agents/raw.txt"), "ignore me")?;

    let assert = Command::new(assert_cmd::cargo::cargo_bin!("context-cli"))
        .args([
            "import",
            "--root",
            root.to_str().unwrap(),
            "--yes",
            "--json",
        ])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    let payload: Value = serde_json::from_str(&stdout)?;
    let items = payload
        .as_array()
        .cloned()
        .expect("expected array of imported documents");

    assert_eq!(items.len(), 2, "should import only markdown files");

    let mut paths: Vec<_> = items.iter().map(|v| v["path"].as_str().unwrap()).collect();
    paths.sort();
    assert_eq!(paths, vec!["agents/plan.md", "notes.md"]);

    for item in &items {
        assert!(item["document"]["key"]
            .as_str()
            .unwrap()
            .starts_with("import/"));
        assert!(item["document"]["body_markdown"]
            .as_str()
            .unwrap()
            .contains('#'));
    }

    Ok(())
}

#[test]
fn import_prompts_for_selection_when_not_yes() -> Result<()> {
    let dir = tempdir()?;
    let root = dir.path();

    fs::write(root.join("agents.md"), "# Agent Notes")?;
    fs::write(root.join("skip.md"), "# Skip")?;

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("context-cli"));
    cmd.args(["import", "--root", root.to_str().unwrap(), "--json"])
        .write_stdin("1\n");

    let assert = cmd.assert().success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    let payload: Value = serde_json::from_str(&stdout)?;
    let items = payload.as_array().cloned().unwrap();

    assert_eq!(items.len(), 1, "should import only selected file");
    assert_eq!(items[0]["path"], "agents.md");

    let stderr = String::from_utf8_lossy(&assert.get_output().stderr).to_lowercase();
    assert!(stderr.contains("select"));

    Ok(())
}
