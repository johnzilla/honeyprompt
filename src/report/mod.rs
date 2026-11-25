use crate::store::{query_report_sessions, query_report_summary};
use chrono::{DateTime, Utc};
use rusqlite::Connection;

/// Escape agent-supplied strings for safe Markdown table interpolation.
///
/// - `|` -> `\|`   (pipe breaks table cells)
/// - `` ` `` -> `` \` ``  (backtick starts code spans)
/// - `\n` -> ` `   (newlines break table rows)
/// - `\r` -> ``    (carriage return stripped)
pub fn md_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '|' => out.push_str(r"\|"),
            '`' => out.push_str(r"\`"),
            '\n' => out.push(' '),
            '\r' => {}
            c => out.push(c),
        }
    }
    out
}

/// Format an epoch-seconds string as a human-readable ISO-8601 datetime (UTC).
///
/// Falls back to the raw string if parsing fails.
pub fn format_timestamp(epoch_str: &str) -> String {
    epoch_str
        .parse::<i64>()
        .ok()
        .and_then(|secs| DateTime::<Utc>::from_timestamp(secs, 0))
        .map(|dt| dt.format("%Y-%m-%d %H:%M UTC").to_string())
        .unwrap_or_else(|| epoch_str.to_string())
}

/// Format a span of seconds compactly: "45s", "12m", "3h 5m", "2d 4h".
pub fn format_duration(secs: i64) -> String {
    if secs < 0 {
        return "—".to_string();
    }
    let (d, h, m, s) = (
        secs / 86_400,
        secs % 86_400 / 3600,
        secs % 3600 / 60,
        secs % 60,
    );
    if d > 0 {
        format!("{d}d {h}h")
    } else if h > 0 {
        format!("{h}h {m}m")
    } else if m > 0 {
        format!("{m}m")
    } else {
        format!("{s}s")
    }
}

/// Most delayed / out-of-band hits listed individually in the report.
const MAX_NOTABLE_HITS: u32 = 200;

/// Render the "Hit Timing" section: per-tier counts by timing label, then the
/// delayed and out-of-band hits individually.
fn hit_timing_section(
    counts: &[crate::store::TimingCounts],
    notable: &[crate::store::NotableHit],
    notable_total: u32,
) -> String {
    use crate::store::{HitTiming, IN_SESSION_WINDOW_SECS, PAGE_VIEW_RETENTION_SECS};
    let mut md = String::new();
    md.push_str("## Hit Timing\n\n");
    md.push_str(&format!(
        "Each hit is compared with page fetches from the same client IP. \
         **in-session**: that IP fetched the page within {} before the hit. \
         **delayed**: it fetched the page earlier than that. \
         **out-of-band**: no fetch from that IP in the previous {}, so whatever read the \
         page is not what fired the callback (a pipeline, a cache, a harvested copy). \
         **unknown**: the hit predates page-fetch recording.\n\n",
        format_duration(IN_SESSION_WINDOW_SECS),
        format_duration(PAGE_VIEW_RETENTION_SECS),
    ));
    md.push_str(
        "These labels describe the network path, not intent. An agent whose browser and \
         fetch tool use different cloud IPs shows as out-of-band even when it acted \
         during its visit, and a client whose IP changes (mobile, CGNAT) can be mislabelled.\n\n",
    );
    if counts.is_empty() {
        md.push_str("No hits stored yet.\n\n");
        return md;
    }
    md.push_str("| Tier | In-session | Delayed | Out-of-band | Unknown |\n");
    md.push_str("|------|------------|---------|-------------|---------|\n");
    for c in counts {
        md.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            c.tier, c.in_session, c.delayed, c.out_of_band, c.unknown
        ));
    }
    md.push('\n');

    if notable.is_empty() {
        md.push_str("No delayed or out-of-band hits.\n\n");
        return md;
    }
    md.push_str("### Delayed and out-of-band hits\n\n");
    md.push_str("| Time | Tier | Nonce | Timing | Since Page Fetch | Since Seeding | Source IP | User Agent | Classification |\n");
    md.push_str("|------|------|-------|--------|------------------|---------------|-----------|------------|----------------|\n");
    for h in notable {
        let since_fetch = match (h.timing, h.last_page_view_at) {
            (HitTiming::OutOfBand, _) | (_, None) => "never".to_string(),
            (_, Some(pv)) => format_duration(h.received_at - pv),
        };
        let since_seed = h
            .seeded_at
            .map(|s| format_duration(h.received_at - s))
            .unwrap_or_else(|| "—".to_string());
        md.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            format_timestamp(&h.received_at.to_string()),
            h.tier,
            md_escape(&h.nonce),
            h.timing.as_str(),
            since_fetch,
            since_seed,
            md_escape(&h.remote_addr),
            md_escape(&h.user_agent),
            md_escape(&h.classification),
        ));
    }
    md.push('\n');
    if notable_total as usize > notable.len() {
        md.push_str(&format!(
            "*Showing the {} most recent of {} delayed or out-of-band hits.*\n\n",
            notable.len(),
            notable_total
        ));
    }
    md
}

/// Most seed hits listed individually in the report.
const MAX_REPORTED_SEED_HITS: u32 = 200;

/// Render the "Seeded Canaries" section: one row per seed, then recent hits.
fn seeded_canaries_section(
    seeds: &[crate::store::SeedSummary],
    hits: &[crate::store::SeedHitRow],
    stored_total: u32,
) -> String {
    let mut md = String::new();
    md.push_str("## Seeded Canaries\n\n");
    md.push_str(
        "Opaque URLs created with `honeyprompt seed add` and planted in private context \
         (system prompts, CLAUDE.md, MCP tool descriptions, RAG documents). A fetch means \
         that content was retained and read or acted on elsewhere; the label says which \
         placement leaked. Known crawlers usually mean the seed ended up somewhere public.\n\n",
    );
    if seeds.is_empty() {
        md.push_str("No seeds. Create one with `honeyprompt seed add <label>`.\n\n");
        return md;
    }
    md.push_str(
        "| Label | Note | Created | Hits | First Hit After Seeding | Last Hit | Distinct IPs |\n",
    );
    md.push_str(
        "|-------|------|---------|------|-------------------------|----------|--------------|\n",
    );
    for s in seeds {
        md.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} |\n",
            md_escape(&s.label),
            md_escape(s.note.as_deref().unwrap_or("—")),
            format_timestamp(&s.created_at.to_string()),
            s.hit_count,
            s.first_hit_at
                .map(|t| format_duration(t - s.created_at))
                .unwrap_or_else(|| "—".to_string()),
            s.last_hit_at
                .map(|t| format_timestamp(&t.to_string()))
                .unwrap_or_else(|| "—".to_string()),
            s.distinct_ips,
        ));
    }
    md.push('\n');
    if hits.is_empty() {
        md.push_str("No seed has been fetched yet.\n\n");
        return md;
    }
    md.push_str("### Seed hits\n\n");
    md.push_str("| Time | Label | Since Seeding | Source IP | User Agent | Classification |\n");
    md.push_str("|------|-------|---------------|-----------|------------|----------------|\n");
    for h in hits {
        md.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} |\n",
            format_timestamp(&h.received_at.to_string()),
            md_escape(&h.label),
            format_duration(h.received_at - h.created_at),
            md_escape(&h.remote_addr),
            md_escape(&h.user_agent),
            md_escape(&h.classification),
        ));
    }
    md.push('\n');
    if stored_total as usize > hits.len() {
        md.push_str(&format!(
            "*Showing the {} most recent of {} stored seed hits.*\n\n",
            hits.len(),
            stored_total
        ));
    }
    md
}

/// Render the "Repeat Hits" section: every stored hit for nonces that fired
/// more than once.
fn repeat_hits_section(hits: &[crate::store::RepeatHit]) -> String {
    let mut md = String::new();
    md.push_str("## Repeat Hits\n\n");
    md.push_str(
        "Every stored hit for nonces that fired more than once. Nonces are baked into \
         the page, so on a public site repeat hits are usually different visitors; the \
         Timing column (see Hit Timing) is what separates in-session use from later use.\n\n",
    );
    if hits.is_empty() {
        md.push_str("No nonce has fired more than once.\n\n");
        return md;
    }
    md.push_str("| Nonce | Tier | Hit | Time | Since Seeding | Timing | Source IP | User Agent | Classification |\n");
    md.push_str("|-------|------|-----|------|---------------|--------|-----------|------------|----------------|\n");
    for h in hits {
        let since_seed = h
            .seeded_at
            .map(|s| format_duration(h.received_at - s))
            .unwrap_or_else(|| "—".to_string());
        md.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            md_escape(&h.nonce),
            h.tier,
            h.hit_number,
            format_timestamp(&h.received_at.to_string()),
            since_seed,
            h.timing.as_str(),
            md_escape(&h.remote_addr),
            md_escape(&h.user_agent),
            md_escape(&h.classification),
        ));
    }
    md.push('\n');
    let mut capped: Vec<(&str, u32, u32)> = hits
        .iter()
        .filter(|h| h.total_fires > h.stored_hits)
        .map(|h| (h.nonce.as_str(), h.stored_hits, h.total_fires))
        .collect();
    capped.dedup();
    for (nonce, stored, total) in capped {
        md.push_str(&format!(
            "*Nonce {} fired {} times; only the first {} hits are stored.*\n\n",
            md_escape(nonce),
            total,
            stored
        ));
    }
    md
}

/// Map tier number to proof level label.
pub fn proof_level(tier: u8) -> &'static str {
    match tier {
        1 => "Arbitrary Callback",
        2 => "Conditional Branch",
        3 => "Computed Callback",
        4 => "Capability Introspection", // NEW Phase 14 (D-14-09)
        5 => "Multi-step Compliance",    // NEW Phase 14 (D-14-09)
        6 => "Context Exfiltration",
        _ => "Unknown",
    }
}

/// Phase 14 (D-14-08, D-14-10, D-14-11, D-14-13): Evidence Table cell for a given
/// session row. T4 renders the full decoded capability list; T5 renders
/// "NNN ✓ VALID" / "NNN ✗ INVALID"; T1-T3 render an em-dash.
/// All agent-supplied strings pass through md_escape (Pattern C, Pitfall 1).
fn evidence_cell(s: &crate::store::ReportSession) -> String {
    match s.tier {
        4 => s
            .t4_capability
            .as_deref()
            .map(md_escape)
            .unwrap_or_else(|| "—".to_string()),
        5 => {
            let proof = s.t5_proof.as_deref().unwrap_or("---");
            match s.t5_proof_valid {
                Some(true) => format!("{} ✓ VALID", md_escape(proof)),
                Some(false) => format!("{} ✗ INVALID", md_escape(proof)),
                None => md_escape(proof),
            }
        }
        _ => "—".to_string(),
    }
}

/// Generate a Markdown disclosure report from the SQLite event store.
///
/// Report structure:
/// - Executive Summary table (session-based counts, per-tier breakdown, date range)
/// - Evidence Table (one row per (session_id, tier) pair for non-crawler sessions)
/// - Known Crawler Sessions section (crawler-only sessions, separated per D-04)
///
/// All agent-supplied strings are passed through md_escape() before table interpolation (Pitfall 1).
/// Timestamps are formatted as human-readable ISO-8601 (not raw epoch seconds).
pub fn generate_report(conn: &Connection) -> anyhow::Result<String> {
    let summary = query_report_summary(conn)?;
    let sessions = query_report_sessions(conn)?;

    let date_range = match (&summary.earliest_event, &summary.latest_event) {
        (Some(earliest), Some(latest)) => {
            format!(
                "{} — {}",
                format_timestamp(earliest),
                format_timestamp(latest)
            )
        }
        _ => "No events recorded".to_string(),
    };

    let mut md = String::new();

    // Title and attribution
    md.push_str("# HoneyPrompt Disclosure Report\n\n");
    md.push_str("*Generated by [HoneyPrompt](https://honeyprompt.dev)*\n\n");

    // Executive Summary
    md.push_str("## Executive Summary\n\n");
    md.push_str("| Metric | Value |\n");
    md.push_str("|--------|-------|\n");
    md.push_str(&format!("| Date Range | {} |\n", date_range));
    md.push_str(&format!(
        "| Total Detection Sessions | {} |\n",
        summary.detection_sessions
    ));
    md.push_str(&format!(
        "| Unique Sessions | {} |\n",
        summary.total_sessions
    ));
    md.push_str(&format!(
        "| Tier 1 (Arbitrary Callback) | {} |\n",
        summary.tier1_sessions
    ));
    md.push_str(&format!(
        "| Tier 2 (Conditional Branch) | {} |\n",
        summary.tier2_sessions
    ));
    md.push_str(&format!(
        "| Tier 3 (Computed Callback) | {} |\n",
        summary.tier3_sessions
    ));
    // Phase 14 (D-14-09, D-14-12): always emit Tier 4/5 rows, even with count=0.
    md.push_str(&format!(
        "| Tier 4 (Capability Introspection) | {} |\n",
        summary.tier4_sessions
    ));
    md.push_str(&format!(
        "| Tier 5 (Multi-step Compliance) | {} |\n",
        summary.tier5_sessions
    ));
    md.push_str(&format!(
        "| Known Crawler Sessions | {} |\n",
        summary.crawler_sessions
    ));
    md.push('\n');

    // Split sessions: detection vs crawler
    let detection_sessions: Vec<_> = sessions
        .iter()
        .filter(|s| !s.classification.starts_with("KnownCrawler"))
        .collect();
    let crawler_sessions: Vec<_> = sessions
        .iter()
        .filter(|s| s.classification.starts_with("KnownCrawler"))
        .collect();

    // Evidence Table (detection sessions only)
    md.push_str("## Evidence Table\n\n");
    md.push_str("| Session | Tier | Proof Level | First Seen | Source IP | User Agent | Fire Count | Classification | Evidence | Payload |\n");
    md.push_str("|---------|------|-------------|------------|-----------|------------|------------|----------------|----------|--------|\n");

    if detection_sessions.is_empty() {
        md.push_str("| — | — | — | — | — | — | — | — | — | — |\n");
    } else {
        for s in &detection_sessions {
            let session_short = md_escape(&s.session_id[..s.session_id.len().min(8)]);
            let tier_str = s.tier.to_string();
            let proof = proof_level(s.tier);
            let first_seen = format_timestamp(&s.first_seen_at);
            let ip = md_escape(&s.remote_addr);
            let ua = md_escape(&s.user_agent);
            let fire = s.fire_count.to_string();
            let class = md_escape(&s.classification);
            let evidence = evidence_cell(s); // NEW Phase 14 (UI-03, UI-04)
            let payload = md_escape(&s.payload_id);
            md.push_str(&format!(
                "| {session_short} | {tier_str} | {proof} | {first_seen} | {ip} | {ua} | {fire} | {class} | {evidence} | {payload} |\n"
            ));
        }
    }
    md.push('\n');

    // Known Crawler Sessions
    md.push_str("## Known Crawler Sessions\n\n");
    md.push_str("| Session | Tier | Proof Level | First Seen | Source IP | User Agent | Fire Count | Classification | Evidence | Payload |\n");
    md.push_str("|---------|------|-------------|------------|-----------|------------|------------|----------------|----------|--------|\n");

    if crawler_sessions.is_empty() {
        md.push_str("| — | — | — | — | — | — | — | — | — | — |\n");
    } else {
        for s in &crawler_sessions {
            let session_short = md_escape(&s.session_id[..s.session_id.len().min(8)]);
            let tier_str = s.tier.to_string();
            let proof = proof_level(s.tier);
            let first_seen = format_timestamp(&s.first_seen_at);
            let ip = md_escape(&s.remote_addr);
            let ua = md_escape(&s.user_agent);
            let fire = s.fire_count.to_string();
            let class = md_escape(&s.classification);
            let evidence = evidence_cell(s); // NEW Phase 14 (UI-03, UI-04)
            let payload = md_escape(&s.payload_id);
            md.push_str(&format!(
                "| {session_short} | {tier_str} | {proof} | {first_seen} | {ip} | {ua} | {fire} | {class} | {evidence} | {payload} |\n"
            ));
        }
    }
    md.push('\n');

    let (notable, notable_total) = crate::store::query_notable_hits(conn, MAX_NOTABLE_HITS)?;
    md.push_str(&hit_timing_section(
        &crate::store::query_hit_timing_counts(conn)?,
        &notable,
        notable_total,
    ));
    md.push_str(&repeat_hits_section(&crate::store::query_repeat_hits(
        conn,
    )?));
    let (seed_hits, seed_hits_total) =
        crate::store::recent_seed_hits(conn, MAX_REPORTED_SEED_HITS)?;
    md.push_str(&seeded_canaries_section(
        &crate::store::list_seeds(conn)?,
        &seed_hits,
        seed_hits_total,
    ));

    md.push_str("---\n");
    md.push_str("*Generated by HoneyPrompt*\n");

    Ok(md)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_md_escape_pipe() {
        assert_eq!(md_escape("pipe|char"), r"pipe\|char");
    }

    #[test]
    fn test_md_escape_newline() {
        assert_eq!(md_escape("new\nline"), "new line");
    }

    #[test]
    fn test_md_escape_backtick() {
        assert_eq!(md_escape("back`tick"), r"back\`tick");
    }

    #[test]
    fn test_md_escape_carriage_return() {
        assert_eq!(md_escape("foo\rbar"), "foobar");
    }

    #[test]
    fn test_format_timestamp_valid_epoch() {
        // 1700000000 = 2023-11-14 22:13 UTC
        let result = format_timestamp("1700000000");
        assert!(result.contains("2023"), "must contain year 2023: {result}");
    }

    #[test]
    fn test_format_timestamp_invalid_falls_back() {
        let result = format_timestamp("not-a-number");
        assert_eq!(result, "not-a-number");
    }

    #[test]
    fn test_proof_level_mapping() {
        assert_eq!(proof_level(1), "Arbitrary Callback");
        assert_eq!(proof_level(2), "Conditional Branch");
        assert_eq!(proof_level(3), "Computed Callback");
        assert_eq!(proof_level(4), "Capability Introspection"); // NEW Phase 14 (D-14-09)
        assert_eq!(proof_level(5), "Multi-step Compliance"); // NEW Phase 14 (D-14-09)
        assert_eq!(proof_level(99), "Unknown");
    }

    // Helper to construct a minimal ReportSession for evidence_cell testing.
    fn make_session(tier: u8) -> crate::store::ReportSession {
        crate::store::ReportSession {
            session_id: "s".to_string(),
            tier,
            payload_id: "p".to_string(),
            embedding_loc: "html_comment".to_string(),
            first_seen_at: "1700000000".to_string(),
            last_seen_at: "1700000000".to_string(),
            fire_count: 1,
            remote_addr: "1.2.3.4".to_string(),
            user_agent: "ua".to_string(),
            classification: "Unknown".to_string(),
            t4_capability: None,
            t5_proof: None,
            t5_proof_valid: None,
        }
    }

    #[test]
    fn test_evidence_cell_t4_some() {
        let mut s = make_session(4);
        s.t4_capability = Some("web_search,browse_page,code_execution".to_string());
        let got = evidence_cell(&s);
        // md_escape over a safe string (no | ` \n \r) is identity — so result equals the input.
        assert_eq!(got, "web_search,browse_page,code_execution");
    }

    #[test]
    fn test_evidence_cell_t4_none_shows_emdash() {
        let s = make_session(4);
        // t4_capability: None — shouldn't happen in practice because a T4 session
        // always has the column populated, but we defend with em-dash.
        assert_eq!(evidence_cell(&s), "—");
    }

    #[test]
    fn test_evidence_cell_t4_escapes_pipe() {
        let mut s = make_session(4);
        s.t4_capability = Some("evil|cap".to_string());
        assert_eq!(evidence_cell(&s), r"evil\|cap");
    }

    #[test]
    fn test_evidence_cell_t5_valid() {
        let mut s = make_session(5);
        s.t5_proof = Some("123".to_string());
        s.t5_proof_valid = Some(true);
        assert_eq!(evidence_cell(&s), "123 ✓ VALID");
    }

    #[test]
    fn test_evidence_cell_t5_invalid() {
        let mut s = make_session(5);
        s.t5_proof = Some("456".to_string());
        s.t5_proof_valid = Some(false);
        assert_eq!(evidence_cell(&s), "456 ✗ INVALID");
    }

    #[test]
    fn test_evidence_cell_t5_unverified() {
        let mut s = make_session(5);
        s.t5_proof = Some("789".to_string());
        s.t5_proof_valid = None;
        // None validity -> just the proof (md_escape'd), no glyph
        assert_eq!(evidence_cell(&s), "789");
    }

    #[test]
    fn test_evidence_cell_t1_t2_t3_emdash() {
        assert_eq!(evidence_cell(&make_session(1)), "—");
        assert_eq!(evidence_cell(&make_session(2)), "—");
        assert_eq!(evidence_cell(&make_session(3)), "—");
    }

    // Regression test (Pitfall 6): header column count must equal separator column
    // count must equal empty-state row column count.
    #[test]
    fn test_evidence_table_column_counts_match() {
        // These strings are the post-Phase-14 10-column literals.
        let header = "| Session | Tier | Proof Level | First Seen | Source IP | User Agent | Fire Count | Classification | Evidence | Payload |";
        let sep = "|---------|------|-------------|------------|-----------|------------|------------|----------------|----------|--------|";
        let empty = "| — | — | — | — | — | — | — | — | — | — |";
        assert_eq!(
            header.matches('|').count(),
            sep.matches('|').count(),
            "header and separator must have matching pipe count"
        );
        assert_eq!(
            header.matches('|').count(),
            empty.matches('|').count(),
            "header and empty-state must have matching pipe count"
        );
    }
}
