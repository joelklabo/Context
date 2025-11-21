use std::path::PathBuf;

#[test]
fn ci_uses_supported_lychee_action_version() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();
    let workflow_path = repo_root.join(".github/workflows/ci.yml");
    let workflow =
        std::fs::read_to_string(&workflow_path).expect("read ci workflow file from repo root");
    assert!(
        workflow.contains("name: Install lychee"),
        "lychee install step missing"
    );
    assert!(
        workflow.contains("if: matrix.os == 'ubuntu-latest'\n        run: cargo install lychee --version 0.21.0 --locked"),
        "lychee install not pinned or missing ubuntu guard"
    );

    assert!(
        workflow.contains("name: Link check (docs + README)"),
        "link check step missing"
    );
    assert!(
        workflow.contains(
            "if: matrix.os == 'ubuntu-latest'\n        run: lychee --verbose --no-progress README.md AGENTS.md AGENTS.README.snippet.md plan.md docs/**/*.md agents/**/*.md"
        ),
        "link check step missing ubuntu guard or expected command"
    );
}
