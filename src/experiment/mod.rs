//! Results from repeated `test-agent` runs (`--trials`, `--results`) and the
//! `honeyprompt experiment summarize` table.
//!
//! Each run is one JSON line (see `Scorecard::result_record`). The summary
//! reports, per agent label, how many runs reached each tier, with a Wilson
//! 95% interval so small samples aren't over-read.

use std::path::Path;

use serde::Deserialize;

use crate::test_agent::{Scorecard, RESULT_SCHEMA};

/// The parts of a `--results` line the summary uses. Unknown fields are ignored.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct TrialRecord {
    #[serde(default)]
    pub schema: u32,
    pub label: String,
    pub tiers: [bool; 5],
    #[serde(default)]
    pub exfil_run: bool,
    #[serde(default)]
    pub exfil_leaked: bool,
    #[serde(default)]
    pub page_fetched: bool,
    #[serde(default)]
    pub agent_timed_out: bool,
    #[serde(default)]
    pub user_agents: Vec<String>,
}

impl TrialRecord {
    /// Build a record from an in-memory scorecard (used for `--trials` text output).
    pub fn from_scorecard(label: &str, card: &Scorecard) -> Self {
        TrialRecord {
            schema: RESULT_SCHEMA,
            label: label.to_string(),
            tiers: card.tiers,
            exfil_run: card.exfil.is_some(),
            exfil_leaked: card.exfil.map(|e| e.leaked).unwrap_or(false),
            page_fetched: card.page_fetched,
            agent_timed_out: card.agent.map(|a| a.timed_out).unwrap_or(false),
            user_agents: card.user_agents.clone(),
        }
    }
}

/// Read a `--results` file; blank lines are skipped, bad lines are errors.
pub fn read_results(path: &Path) -> anyhow::Result<Vec<TrialRecord>> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("cannot read {}: {e}", path.display()))?;
    parse_results(&text).map_err(|e| anyhow::anyhow!("{}: {e}", path.display()))
}

/// Parse JSON lines into records.
pub fn parse_results(text: &str) -> anyhow::Result<Vec<TrialRecord>> {
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let r: TrialRecord = serde_json::from_str(line)
            .map_err(|e| anyhow::anyhow!("line {}: not a test-agent result: {e}", i + 1))?;
        if r.schema > RESULT_SCHEMA {
            anyhow::bail!(
                "line {}: result schema {} is newer than this honeyprompt understands ({RESULT_SCHEMA})",
                i + 1,
                r.schema
            );
        }
        out.push(r);
    }
    Ok(out)
}

/// Wilson score interval at 95% for `k` successes in `n` trials, as (low, high)
/// fractions. Plain arithmetic; `n == 0` gives (0, 1).
pub fn wilson_95(k: u32, n: u32) -> (f64, f64) {
    if n == 0 {
        return (0.0, 1.0);
    }
    const Z: f64 = 1.959_963_984_540_054;
    let n = f64::from(n);
    let p = f64::from(k) / n;
    let z2 = Z * Z;
    let denom = 1.0 + z2 / n;
    let center = (p + z2 / (2.0 * n)) / denom;
    let half = Z * (p * (1.0 - p) / n + z2 / (4.0 * n * n)).sqrt() / denom;
    ((center - half).max(0.0), (center + half).min(1.0))
}

fn cell(k: u32, n: u32) -> String {
    if n == 0 {
        return "—".to_string();
    }
    let (lo, hi) = wilson_95(k, n);
    format!(
        "{k}/{n} ({:.0}–{:.0}%)",
        (lo * 100.0).round(),
        (hi * 100.0).round()
    )
}

/// Make text safe inside a Markdown table cell or code span.
fn md(text: &str) -> String {
    text.replace('|', "\\|")
        .replace('`', "'")
        .chars()
        .filter(|c| !c.is_control())
        .collect()
}

struct AgentStats<'a> {
    label: &'a str,
    runs: u32,
    fetched: u32,
    tiers: [u32; 5],
    any: u32,
    exfil_runs: u32,
    leaks: u32,
    timed_out: u32,
    user_agents: Vec<&'a str>,
}

fn group(records: &[TrialRecord]) -> Vec<AgentStats<'_>> {
    let mut out: Vec<AgentStats> = Vec::new();
    for r in records {
        let idx = match out.iter().position(|a| a.label == r.label) {
            Some(i) => i,
            None => {
                out.push(AgentStats {
                    label: &r.label,
                    runs: 0,
                    fetched: 0,
                    tiers: [0; 5],
                    any: 0,
                    exfil_runs: 0,
                    leaks: 0,
                    timed_out: 0,
                    user_agents: Vec::new(),
                });
                out.len() - 1
            }
        };
        let a = &mut out[idx];
        a.runs += 1;
        a.fetched += u32::from(r.page_fetched);
        for (t, hit) in r.tiers.iter().enumerate() {
            a.tiers[t] += u32::from(*hit);
        }
        a.any += u32::from(r.tiers.iter().any(|&h| h));
        a.exfil_runs += u32::from(r.exfil_run);
        a.leaks += u32::from(r.exfil_run && r.exfil_leaked);
        a.timed_out += u32::from(r.agent_timed_out);
        for ua in &r.user_agents {
            if !a.user_agents.contains(&ua.as_str()) {
                a.user_agents.push(ua);
            }
        }
    }
    out
}

/// Markdown summary: one row per agent label, in first-seen order.
pub fn render_summary(records: &[TrialRecord]) -> String {
    if records.is_empty() {
        return "No results.\n".to_string();
    }
    let stats = group(records);
    let mut out = String::from(
        "| Agent | Runs | Loaded page | T1 | T2 | T3 | T4 | T5 | Any T1–T5 | T6 leaked |\n\
         |---|---|---|---|---|---|---|---|---|---|\n",
    );
    for a in &stats {
        out.push_str(&format!(
            "| {} | {} | {}/{} | {} | {} | {} | {} | {} | {} | {} |\n",
            md(a.label),
            a.runs,
            a.fetched,
            a.runs,
            cell(a.tiers[0], a.runs),
            cell(a.tiers[1], a.runs),
            cell(a.tiers[2], a.runs),
            cell(a.tiers[3], a.runs),
            cell(a.tiers[4], a.runs),
            cell(a.any, a.runs),
            cell(a.leaks, a.exfil_runs),
        ));
    }
    out.push_str(
        "\nEach cell is runs that reached the tier / total runs, with a Wilson 95% interval. \
         T6 counts only runs made with `--exfil` (— when none).\n",
    );

    let timed_out: Vec<String> = stats
        .iter()
        .filter(|a| a.timed_out > 0)
        .map(|a| format!("{} ({}/{})", md(a.label), a.timed_out, a.runs))
        .collect();
    if !timed_out.is_empty() {
        out.push_str(&format!(
            "\nRuns where the agent was killed at the timeout: {}.\n",
            timed_out.join(", ")
        ));
    }
    let unfetched: Vec<String> = stats
        .iter()
        .filter(|a| a.fetched < a.runs)
        .map(|a| format!("{} ({}/{})", md(a.label), a.runs - a.fetched, a.runs))
        .collect();
    if !unfetched.is_empty() {
        out.push_str(&format!(
            "\nRuns where nothing loaded the page (a 0 there means \"never saw it\", not \"refused\"): {}.\n",
            unfetched.join(", ")
        ));
    }

    out.push_str("\n### User agents seen\n\n");
    for a in &stats {
        if a.user_agents.is_empty() {
            out.push_str(&format!("- **{}**: none\n", md(a.label)));
        } else {
            let uas: Vec<String> = a
                .user_agents
                .iter()
                .map(|u| format!("`{}`", md(u)))
                .collect();
            out.push_str(&format!("- **{}**: {}\n", md(a.label), uas.join(", ")));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rec(label: &str, tiers: [bool; 5]) -> TrialRecord {
        TrialRecord {
            schema: 1,
            label: label.into(),
            tiers,
            exfil_run: false,
            exfil_leaked: false,
            page_fetched: true,
            agent_timed_out: false,
            user_agents: vec!["UA/1".into()],
        }
    }

    #[test]
    fn wilson_matches_known_values() {
        // Reference values (Wilson, z = 1.96): 7/10 -> 0.3968..0.8922,
        // 0/10 -> 0..0.2775, 10/10 -> 0.7225..1.
        let (lo, hi) = wilson_95(7, 10);
        assert!(
            (lo - 0.3968).abs() < 1e-3 && (hi - 0.8922).abs() < 1e-3,
            "{lo} {hi}"
        );
        let (lo, hi) = wilson_95(0, 10);
        assert!(lo == 0.0 && (hi - 0.2775).abs() < 1e-3, "{hi}");
        let (lo, hi) = wilson_95(10, 10);
        assert!(
            (lo - 0.7225).abs() < 1e-3 && (hi - 1.0).abs() < 1e-9,
            "{lo}"
        );
        assert_eq!(wilson_95(0, 0), (0.0, 1.0));
    }

    #[test]
    fn summary_groups_by_label_in_order() {
        let mut r = vec![
            rec("browser-use", [true, false, false, false, false]),
            rec("curl", [false; 5]),
            rec("browser-use", [true, true, false, false, false]),
        ];
        r[1].page_fetched = false;
        r[1].user_agents.clear();
        r[2].exfil_run = true;
        r[2].exfil_leaked = true;
        let text = render_summary(&r);
        let bu = text.find("| browser-use | 2 |").expect(&text);
        let curl = text.find("| curl | 1 |").expect(&text);
        assert!(bu < curl);
        assert!(
            text.contains("| browser-use | 2 | 2/2 | 2/2 (34–100%) | 1/2 "),
            "{text}"
        );
        assert!(
            text.contains("| 1/1 (21–100%) |"),
            "T6 out of exfil runs only: {text}"
        );
        assert!(text.contains("| curl | 1 | 0/1 |"), "{text}");
        assert!(text.contains("| — |"), "no exfil runs for curl");
        assert!(text.contains("never saw it"), "{text}");
        assert!(text.contains("- **curl**: none"));
        assert!(text.contains("`UA/1`"));
    }

    #[test]
    fn markdown_is_escaped() {
        let mut r = rec("a|b", [false; 5]);
        r.user_agents = vec!["evil`|ua\n".into()];
        let text = render_summary(&[r]);
        assert!(text.contains("| a\\|b |"), "{text}");
        assert!(text.contains("`evil'\\|ua`"), "{text}");
    }

    #[test]
    fn parse_skips_blank_lines_and_rejects_garbage() {
        let line = serde_json::json!({
            "schema": 1, "label": "x", "tiers": [true, false, false, false, false],
            "extra_field": 42
        })
        .to_string();
        let parsed = parse_results(&format!("{line}\n\n{line}\n")).unwrap();
        assert_eq!(parsed.len(), 2);
        assert!(parsed[0].tiers[0]);
        let err = parse_results("{not json}\n").unwrap_err().to_string();
        assert!(err.contains("line 1"), "{err}");
        let future = r#"{"schema": 99, "label": "x", "tiers": [false,false,false,false,false]}"#;
        assert!(parse_results(future).is_err());
        assert_eq!(render_summary(&[]), "No results.\n");
    }
}
