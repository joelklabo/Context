use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::process::Command;

#[derive(Debug)]
struct Task {
    id: String,
    owner: Option<String>,
    status: Option<String>,
    raw_status: Option<String>,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("plan-check: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let contents =
        fs::read_to_string("plan.md").map_err(|e| format!("failed to read plan.md: {e}"))?;

    let task_re = Regex::new(r"^- \[( |x)\]\s+([a-z0-9-]+):").map_err(|e| e.to_string())?;
    let owner_re = Regex::new(r"@owner\(([^)]+)\)").map_err(|e| e.to_string())?;
    let status_re = Regex::new(r"@status\(([^)]+)\)").map_err(|e| e.to_string())?;

    let mut tasks: Vec<Task> = Vec::new();
    let mut current_index: Option<usize> = None;

    // Track whether we're inside a fenced code block (``` ... ```).
    // Lines inside code fences are ignored for task parsing so examples don't trip validation.
    let mut in_code_block = false;

    for line in contents.lines() {
        let trimmed = line.trim_start();

        // Toggle code block state on lines starting with ```
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            // When entering or leaving a code block, do not associate this line with a task.
            current_index = None;
            continue;
        }

        // Skip any content inside fenced code blocks
        if in_code_block {
            continue;
        }

        if let Some(caps) = task_re.captures(line) {
            let id = caps.get(2).unwrap().as_str().to_string();
            tasks.push(Task {
                id,
                owner: None,
                status: None,
                raw_status: None,
            });
            current_index = Some(tasks.len() - 1);
        } else if trimmed.starts_with('@') || line.contains("@owner(") || line.contains("@status(")
        {
            if let Some(idx) = current_index {
                let t = &mut tasks[idx];
                if let Some(caps) = owner_re.captures(line) {
                    t.owner = Some(caps.get(1).unwrap().as_str().to_string());
                }
                if let Some(caps) = status_re.captures(line) {
                    let raw = caps.get(1).unwrap().as_str().to_string();
                    t.raw_status = Some(raw.clone());
                    // status canonical form: first token before comma
                    let parts: Vec<_> = raw.split(',').collect();
                    t.status = Some(parts[0].trim().to_string());
                }
            }
        } else {
            current_index = None;
        }
    }

    let mut errors: Vec<String> = Vec::new();

    // Basic invariants per task
    for t in &tasks {
        if t.owner.is_none() {
            errors.push(format!("task {} missing @owner(...)", t.id));
        }
        if t.status.is_none() {
            errors.push(format!("task {} missing @status(...)", t.id));
        }

        if let (Some(owner), Some(status)) = (&t.owner, &t.status) {
            if status == "in-progress" && owner == "unassigned" {
                errors.push(format!(
                    "task {} is in-progress but @owner(unassigned)",
                    t.id
                ));
            }
            if status == "unclaimed" && owner != "unassigned" {
                errors.push(format!("task {} is unclaimed but owner is {}", t.id, owner));
            }
        }

        // For done status, check commit field is present and git knows it
        if let Some(raw) = &t.raw_status {
            if raw.starts_with("done") {
                // expect "done,commit=<hash>"
                let has_commit = raw.contains("commit=");
                if !has_commit {
                    errors.push(format!(
                        "task {} @status(done,...) must include commit=<hash>",
                        t.id
                    ));
                } else if let Some(commit_idx) = raw.find("commit=") {
                    let after = &raw[commit_idx + "commit=".len()..];
                    let hash = after.split([')', ',']).next().unwrap().trim();
                    if !hash.is_empty() && hash != "<bootstrap>" {
                        let ok = Command::new("git")
                            .args(["rev-parse", "--verify", hash])
                            .output()
                            .map(|o| o.status.success())
                            .unwrap_or(true);
                        if !ok {
                            errors.push(format!(
                                "task {} refers to unknown commit hash {}",
                                t.id, hash
                            ));
                        }
                    }
                }
            }
        }
    }

    // Ensure each owner has at most one in-progress task
    let mut owner_in_progress: HashMap<String, Vec<String>> = HashMap::new();
    for t in &tasks {
        if let (Some(owner), Some(status)) = (&t.owner, &t.status) {
            if status == "in-progress" {
                owner_in_progress
                    .entry(owner.clone())
                    .or_default()
                    .push(t.id.clone());
            }
        }
    }
    for (owner, ids) in owner_in_progress {
        if ids.len() > 1 {
            errors.push(format!(
                "owner {} has multiple in-progress tasks: {}",
                owner,
                ids.join(", ")
            ));
        }
    }

    if errors.is_empty() {
        println!("plan-check: OK ({} tasks validated)", tasks.len());
        Ok(())
    } else {
        eprintln!("plan-check: {} issue(s) found:", errors.len());
        for e in errors {
            eprintln!("  - {e}");
        }
        Err("plan.md validation failed".into())
    }
}
