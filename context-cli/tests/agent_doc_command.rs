use anyhow::Result;
use assert_cmd::Command;

#[test]
fn agent_doc_emits_detailed_markdown() -> Result<()> {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("context-cli"));
    let assert = cmd
        .args(["agent-doc", "--format", "markdown"])
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone())?;

    assert!(
        stdout.contains("# context - Agent Usage"),
        "should include the document heading"
    );
    assert!(
        stdout.contains("Quickstart"),
        "should include a quickstart section"
    );
    assert!(
        stdout.contains("Command cheatsheet"),
        "should include a command cheatsheet section"
    );
    assert!(
        !stdout.contains("This is a stub"),
        "should not emit the old stub content"
    );

    Ok(())
}
