use chrono::{DateTime, Duration, Utc};
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
    scenario: Option<String>,
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
    let now = Utc::now();

    let (task_count, errors) = validate_plan(&contents, now);
    if errors.is_empty() {
        println!("plan-check: OK ({} tasks validated)", task_count);
        return Ok(());
    }

    eprintln!("plan-check: {} issue(s) found:", errors.len());
    for e in errors {
        eprintln!("  - {e}");
    }
    Err("plan.md validation failed".into())
}

fn validate_plan(contents: &str, now: DateTime<Utc>) -> (usize, Vec<String>) {
    let task_re = Regex::new(r"^- \[( |x)\]\s+([a-z0-9-]+):").expect("compile task regex");
    let owner_re = Regex::new(r"@owner\(([^)]+)\)").expect("compile owner regex");
    let status_re = Regex::new(r"@status\(([^)]+)\)").expect("compile status regex");
    let scenario_re = Regex::new(r"@scenario\(([^)]+)\)").expect("compile scenario regex");

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
                scenario: None,
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
                if let Some(caps) = scenario_re.captures(line) {
                    t.scenario = Some(caps.get(1).unwrap().as_str().to_string());
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

            if status == "in-progress" {
                if let Some(scenario) = &t.scenario {
                    if let Some(ts) = parse_scenario_timestamp(scenario) {
                        let age = now.signed_duration_since(ts);
                        if age > Duration::minutes(stale_timeout_minutes()) {
                            errors.push(format!(
                                "task {} in-progress scenario {} is older than {} minutes; release or refresh the task",
                                t.id,
                                scenario,
                                stale_timeout_minutes()
                            ));
                        }
                    } else {
                        errors.push(format!(
                            "task {} in-progress has @scenario({}) without a parsable timestamp for stale timeout check",
                            t.id, scenario
                        ));
                    }
                } else {
                    errors.push(format!(
                        "task {} in-progress missing @scenario(...) timestamp for stale timeout check",
                        t.id
                    ));
                }
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

    (tasks.len(), errors)
}

fn parse_scenario_timestamp(scenario: &str) -> Option<DateTime<Utc>> {
    // Scenario values start with an RFC3339 timestamp; keep everything through the trailing Z.
    let end = scenario.find('Z')?;
    let ts = &scenario[..=end];
    DateTime::parse_from_rfc3339(ts)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

fn stale_timeout_minutes() -> i64 {
    const DEFAULT: i64 = 20;
    match std::env::var("PLAN_STALE_MINUTES") {
        Ok(v) => v.parse::<i64>().unwrap_or(DEFAULT),
        Err(_) => DEFAULT,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::SecondsFormat;

    fn build_plan(task_line: &str, metadata: &str) -> String {
        format!("# plan\n\n- [ ] {task_line}\n      {metadata}\n")
    }

    #[test]
    fn flags_stale_in_progress_tasks() {
        let now = Utc::now();
        let stale_ts = (now - Duration::minutes(30)).to_rfc3339_opts(SecondsFormat::Secs, true);
        let plan = build_plan(
            "cli-999: dummy task",
            &format!(
                "@area(cli) @owner(context-cli-agent) @status(in-progress) @scenario({}-cli-999)",
                stale_ts
            ),
        );

        let (_, errors) = validate_plan(&plan, now);
        assert!(
            errors
                .iter()
                .any(|e| e.contains("cli-999") && e.contains("older than")),
            "expected stale error, got: {:?}",
            errors
        );
    }

    #[test]
    fn accepts_recent_in_progress_tasks() {
        let now = Utc::now();
        let fresh_ts = (now - Duration::minutes(5)).to_rfc3339_opts(SecondsFormat::Secs, true);
        let plan = build_plan(
            "cli-123: fresh task",
            &format!(
                "@area(cli) @owner(context-cli-agent) @status(in-progress) @scenario({}-cli-123)",
                fresh_ts
            ),
        );

        let (_, errors) = validate_plan(&plan, now);
        assert!(
            errors.is_empty(),
            "expected no errors for fresh task, got: {:?}",
            errors
        );
    }
}
