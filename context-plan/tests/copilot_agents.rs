use std::fs;
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    // CARGO_MANIFEST_DIR points at context-plan/, so walk up to workspace root.
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

fn read_copilot_agent_files() -> Vec<String> {
    let dir = workspace_root().join(".github/agents");
    assert!(
        dir.is_dir(),
        "{} directory should exist for Copilot agent definitions",
        dir.display()
    );

    let mut files: Vec<_> = fs::read_dir(dir)
        .expect("failed to read .github/agents")
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                path.to_str().map(|s| s.to_string())
            } else {
                None
            }
        })
        .collect();

    files.sort();
    files
}

#[test]
fn copilot_agents_have_expected_structure() {
    let files = read_copilot_agent_files();
    assert!(
        !files.is_empty(),
        "no Copilot agent definitions found in .github/agents"
    );

    let required_sections = [
        "## Scope",
        "## Responsibilities",
        "## Allowed actions",
        "## Forbidden actions",
        "## Workflow",
    ];

    for file in files {
        let contents =
            fs::read_to_string(&file).unwrap_or_else(|_| panic!("failed to read {}", file));

        // Minimal YAML front matter check for Copilot CLI agent metadata.
        let mut parts = contents.splitn(3, "---");
        parts.next(); // leading empty part before first fence
        let front_matter = parts
            .next()
            .unwrap_or_else(|| panic!("{} missing YAML front matter", file));
        assert!(
            front_matter.contains("name:"),
            "{} front matter missing name:",
            file
        );
        assert!(
            front_matter.contains("description:"),
            "{} front matter missing description:",
            file
        );

        for section in &required_sections {
            assert!(
                contents.contains(section),
                "{} missing section {}",
                file,
                section
            );
        }
    }
}
