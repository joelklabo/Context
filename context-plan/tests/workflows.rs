use regex::Regex;
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
    let re = Regex::new(
        r"(?s)- name: Link check.*?\n\s+if: matrix\.os == 'ubuntu-latest'\n\s+uses: lycheeverse/lychee-action@(?P<ver>\S+).*?args:\s*--verbose --no-progress \.",
    )
    .expect("compile link-check regex");

    let caps = re
        .captures(&workflow)
        .expect("lychee action reference not found");
    let version = caps.name("ver").unwrap().as_str();

    assert_eq!(
        version, "v2.7.0",
        "Update expected lychee-action version when bumping CI link checker"
    );
}
