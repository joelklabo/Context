use std::fs;
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

fn read_root_file(path: &str) -> String {
    let full_path = workspace_root().join(path);
    fs::read_to_string(&full_path)
        .unwrap_or_else(|_| panic!("failed to read {}", full_path.display()))
}

fn quoted_agent_doc() -> String {
    let doc = read_root_file("docs/agent-doc.md");
    doc.lines()
        .map(|line| {
            if line.is_empty() {
                ">".to_string()
            } else {
                format!("> {}", line)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn agent_doc_snippets_are_in_sync() {
    let expected = quoted_agent_doc();
    for file in ["AGENTS.md", "CLAUDE.md"] {
        let contents = read_root_file(file);
        assert!(
            contents.contains(&expected),
            "{file} should include the agent-doc snippet from docs/agent-doc.md; run `cargo run -p context-cli -- agent-doc --format markdown > docs/agent-doc.md` to refresh"
        );
    }
}
