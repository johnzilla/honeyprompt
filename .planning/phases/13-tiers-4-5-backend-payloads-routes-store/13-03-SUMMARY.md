---
phase: 13-tiers-4-5-backend-payloads-routes-store
plan: 03
subsystem: generator
tags: [rust, minijinja, generator, tier4, tier5, seed-derivation, json-ld, template]

# Dependency graph
requires:
  - phase: 13-01-foundation-catalog-types-helpers
    provides: Tier::Tier4/Tier5 enum variants, T5Formula, Payload.t5_formula, nonce::derive_seed, nonce::generate_nonce, tier4.toml + tier5.toml catalogs
provides:
  - Real Tier4 render arm in generator::generate — substitutes {callback_url_b64_base} → <base>/cb/v4/<nonce>, persists nonce_map row, pushes NonceMapping + RenderedPayload
  - Real Tier5 render arm in generator::generate — substitutes {callback_url_proof_base} → <base>/cb/v5/<nonce>, derives seed via nonce::derive_seed, emits one <script type="application/ld+json">{"verification_seed":<u32>,"nonce":"<hex>"}</script> block per T5 payload
  - `t5_seed_scripts: Vec<String>` accumulator + post-loop join into `seed_scripts_json` local
  - New `seed_scripts_json` template context key wired into the `context!` for index.html.jinja (and ONLY that template)
  - Single new `{{ seed_scripts_json | safe }}` block inside <head> of assets/templates/index.html.jinja
  - 4 new generator unit tests covering T4 b64-base rendering, T5 proof-base rendering, T5 seed JSON-LD emission count + value correctness, and the no-T5 zero-seed-block invariant
affects: [13-04-server-routes-t4-t5, 14-monitor-report, 15-test-agent-docs]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Template context extension pattern: add one new key to the `context!` macro invocation at the one call site for the target template; add ONE `{{ key | safe }}` block in the template; no conditional wrapping when an empty string is the natural zero-value"
    - "Accumulator + post-loop join for per-match-arm HTML fragments: collect strings in a Vec<String> during the tier dispatch loop, join after the loop, pass through as a single template key (keeps the loop body tier-local and the template invocation flat)"
    - "Panic-as-invariant documentation: nonce::derive_seed(&nonce).expect(...) inside the generator is explicitly documented as a generator precondition (generate_nonce() always produces 16-char hex → derive_seed returns Some); any future invariant break panics at build time rather than silently emitting a wrong seed"

key-files:
  created: []
  modified:
    - src/generator/mod.rs (+164 net lines — 2 new match arms, accumulator + join, 1 new context! key, 4 new unit tests + 1 helper fn)
    - assets/templates/index.html.jinja (+1 line — one `{{ seed_scripts_json | safe }}` block just before </head>)

key-decisions:
  - "Self-identifying seed blocks (Q1 resolution): each emitted JSON-LD block carries both \"verification_seed\" and \"nonce\" fields so agents reading multiple blocks can correlate; the server ignores \"nonce\" in the block body at verification time and uses the URL-path nonce as the authoritative key, so the extra field cannot alter server-side verification semantics"
  - "Single new template key, no conditional wrapper: empty string renders to empty string natively in minijinja; adding {% if seed_scripts_json %} would introduce Jinja control flow the existing template avoids, and the byte-identical case (when no T5 active) is preserved without it"
  - "`| safe` filter is correct here: the accumulator's strings are built from format!() with a u32 seed and a 16-char hex nonce — both trusted internal values — so disabling HTML escaping is safe; threat model T-13-XSS-SEED documents this invariant so any future change that pipes external data into the accumulator must re-examine it"
  - "Panic documentation vs defensive handling: used `.expect(...)` rather than `.ok_or(...)?` because `generate_nonce()` is a generator precondition (always 16-char hex); documenting the invariant via panic makes any future break loud rather than silently emitting a wrong seed in production HTML"

patterns-established:
  - "Tier match arm extension: Tier4 arm clones the Tier3 arm structure (nonce → callback URL → placeholder substitution → insert_nonce → NonceMapping + RenderedPayload push); Tier5 arm adds one extra step (derive_seed + push to t5_seed_scripts) but follows the same skeleton — keeps dispatch predictable"
  - "Template accumulator pattern: per-tier-arm-specific HTML emission goes into a Vec<String>, joined once after the main loop, passed as a single template key; avoids per-arm template rendering and keeps the template untouched by tier dispatch logic"

requirements-completed: [PAYLOAD-04]

# Metrics
duration: ~3min
completed: 2026-04-24
---

# Phase 13 Plan 03: Generator — T4/T5 Render + Seed JSON-LD Summary

**Replaced Plan 01's no-op Tier4/Tier5 stub with real render logic: T4 substitutes {callback_url_b64_base} to `<base>/cb/v4/<nonce>`, T5 substitutes {callback_url_proof_base} to `<base>/cb/v5/<nonce>` and emits one `<script type="application/ld+json">{"verification_seed":<u32>,"nonce":"<hex>"}</script>` block per T5 payload via a single new `seed_scripts_json` template key rendered with `{{ ... | safe }}` inside `<head>` — 4 new unit tests green, all 11 existing `tests/test_generate.rs` integration tests unchanged and still pass (D-13-18 regression guard intact).**

## Performance

- **Duration:** ~3 min
- **Started:** 2026-04-24T17:12:40Z
- **Completed:** 2026-04-24T17:15:32Z
- **Tasks:** 1 (TDD: split into RED + GREEN commits)
- **Files modified:** 2 (src/generator/mod.rs, assets/templates/index.html.jinja)

## Accomplishments

- Replaced (not duplicated) Plan 01's `Tier::Tier4 | Tier::Tier5 => { /* skipped */ }` stub in `src/generator/mod.rs::generate` with two full match arms that render T4 and T5 payloads end-to-end (nonce generation, callback URL construction, placeholder substitution, nonce_map persistence, NonceMapping + RenderedPayload emission).
- Implemented D-13-05 seed JSON-LD emission: each active T5 payload contributes one `<script type="application/ld+json">{"verification_seed":N,"nonce":"H"}</script>` block. Seed `N` is `nonce::derive_seed(&nonce).expect(...)` (u32 from first 8 hex chars of the 16-char CSPRNG nonce). The server recomputes the same seed at T5 verification time from the URL-path nonce.
- Wired a single new `seed_scripts_json` template context key into the `context!` invocation at the index.html.jinja call site; the other three templates (robots.txt.jinja, ai.txt.jinja, security.txt.jinja) are untouched.
- Added one `{{ seed_scripts_json | safe }}` line just before `</head>` in `assets/templates/index.html.jinja` — matches the existing `| safe` idiom used for the five per-embedding-location loops (lines 14, 26, 36, 41, 45 of the template). No conditional wrapper — the empty string case is naturally a zero-byte render.
- Resolved RESEARCH Q1 (seed ambiguity when multiple T5 payloads coexist): each block carries a `"nonce"` field so agents reading multiple blocks can correlate; server ignores the body field and uses the URL-path nonce at verification time.
- Added 4 new tests in `src/generator/tests`: `test_tier4_renders_with_b64_base`, `test_tier5_renders_with_proof_base`, `test_t5_seed_json_ld_emission` (asserts block count == T5 payload count AND each block's seed matches `derive_seed` of the corresponding nonce AND each block's nonce field matches a T5 mapping), `test_no_seed_block_when_t5_filtered_out` (regression guard: tiers=[1,2,3] produces zero `verification_seed` substrings).
- All 11 existing integration tests in `tests/test_generate.rs` pass unmodified — D-13-18 regression guard is intact for the /cb/v1/ T1-T3 byte-identical path.

## Task Commits

1. **Task 1 RED: failing tests for T4/T5 generator + seed JSON-LD** — `dd4f105` (test)
2. **Task 1 GREEN: render T4/T5 payloads + emit seed JSON-LD blocks** — `df3ad2b` (feat; replaces Plan 01 stub)

_No refactor commit needed — GREEN implementation was clean (clippy + fmt passed after one trivial `Config::default()` struct-update-syntax tweak applied during GREEN before commit)._

## Files Created/Modified

- `src/generator/mod.rs` (+164 net lines)
  - New line-53-area: `let mut t5_seed_scripts: Vec<String> = Vec::new();` accumulator declaration with D-13-05 rationale comment
  - New `Tier::Tier4 =>` arm (replaces old line-152 no-op stub, shares structure with existing `Tier::Tier3 =>` arm at line 124) — builds `<base>/cb/v4/<nonce>` URL, replaces `{callback_url_b64_base}`, inserts nonce, pushes NonceMapping + RenderedPayload
  - New `Tier::Tier5 =>` arm — builds `<base>/cb/v5/<nonce>` URL, replaces `{callback_url_proof_base}`, calls `nonce::derive_seed(&nonce).expect(...)` (documented as generator precondition), pushes self-identifying `<script type="application/ld+json">{...}</script>` string onto `t5_seed_scripts`, inserts nonce, pushes NonceMapping + RenderedPayload
  - After the main loop: `let seed_scripts_json = t5_seed_scripts.join("\n");`
  - Extended `context!` invocation at the index.html.jinja call site with `seed_scripts_json => &seed_scripts_json,` (the only `context!` changed; robots.txt.jinja, ai.txt.jinja, security.txt.jinja are untouched)
  - 4 new `#[test]` functions + `test_config_with_tiers(Vec<u8>) -> Config` helper inside `#[cfg(test)] mod tests`
- `assets/templates/index.html.jinja` (+1 line)
  - Single new line `  {{ seed_scripts_json | safe }}` between line 15 `{% endif %}{% endfor %}` (end of meta_tag block) and line 16 `</head>`; no existing template line removed or edited

## JSON-LD Block Format (Emitted)

Exact format emitted per T5 payload:

```html
<script type="application/ld+json">{"verification_seed":2864434397,"nonce":"abcdef12abcdef12"}</script>
```

- `verification_seed` is a decimal-formatted u32 (range `0..=u32::MAX`), computed as `u32::from_str_radix(&nonce[0..8], 16).unwrap()`.
- `nonce` is the full 16-char lowercase hex nonce from `generate_nonce()`.
- Blocks are joined with `"\n"` and rendered via a single `{{ seed_scripts_json | safe }}` statement — no Jinja conditional wrapper, no per-block iteration in the template.

## Decisions Made

- **Single new template key + unconditional render** (committed approach from the plan): chose `{{ seed_scripts_json | safe }}` without an `{% if %}` wrapper. Rationale: minijinja renders an empty string to zero bytes naturally, so the no-T5 case is byte-preserved without Jinja control flow (the existing template deliberately uses only `{% for %}{% if %}` pairs for payload iteration and never bare `{% if var %}` for presence checks). Keeps the template idiom consistent.
- **Self-identifying seed blocks** (Q1 resolution, D-13-05 refinement): each JSON-LD block includes both `"verification_seed"` and `"nonce"` fields. The server uses the URL-path nonce as the authoritative key at verification time and ignores the body's `"nonce"` field, so the extra field cannot alter verification semantics (documented in threat register T-13-04). Benefits: agents reading multiple blocks can correlate without semantic guessing.
- **`.expect(...)` for seed derivation inside the generator**: `nonce::generate_nonce()` always returns a well-formed 16-char hex string, so `derive_seed` returns `Some(_)`. Using `.expect("generator-produced nonce is well-formed 16-char hex")` documents this as a generator precondition. Any future invariant break (e.g., someone changes `generate_nonce` to produce 15 chars) panics loudly at build/test time rather than silently emitting a wrong seed in production HTML. The panic stays off the untrusted-input boundary — the server's T5 handler uses `derive_seed(nonce).ok_or_else(...)` with `.ok()`-style defensive handling against URL-path input, preserving T-13-01 non-panic guarantee at the handler boundary.
- **Accumulator + join vs per-arm template rendering**: collected all T5 blocks in a `Vec<String>`, joined with `"\n"` after the loop. Alternatives considered but rejected: (a) passing the full vector into the template and iterating there — would require a new `{% for seed in seed_scripts %}` loop and expose raw `<script>` tags to Jinja's escape logic, requiring `| safe` on each field; (b) pre-rendering a sub-template — more moving parts for no benefit. The join approach lets the template stay idiomatic.

## Deviations from Plan

None — plan executed exactly as written.

The one adjustment made during GREEN (switching the test helper `test_config_with_tiers` from `let mut c = Config::default(); c.tiers = tiers; c` to struct-update-syntax `Config { tiers, ..Config::default() }`) was required to silence `clippy::field_reassign_with_default`, which the plan's acceptance criteria explicitly mandate via `cargo clippy --all-targets --lib -- -D warnings`. Not a deviation — a direct application of the plan's clippy-clean criterion.

## Issues Encountered

- **cargo test CLI flag ambiguity.** Initial attempt to run all four new tests in one invocation (`cargo test --lib test1 test2 test3 test4`) failed with `error: unexpected argument 'test2' found` — cargo test only takes one positional TESTNAME argument. Worked around by running tests by common prefix (`cargo test --lib generator::tests::test_tier`, then `::test_t5`, then `::test_no_seed`). No behavior impact; just a CLI ergonomics note.
- **rustfmt canonical formatting.** `cargo fmt --check` initially flagged the T4 arm's `format!("{}/cb/v4/{}", ...)` as multi-line when rustfmt preferred single-line. Ran `cargo fmt` (not `--check`) once to apply canonical formatting; subsequent `--check` passed. No behavior change.

## Known Stubs

None. Plan 01's Tier4/Tier5 no-op match arm in `src/generator/mod.rs` was the only stub from prior work; this plan replaces it with real render logic. The accumulator, join, context key, and template line are all live and exercised by tests.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

Plan 04 (server T4/T5 routes) can proceed. Verified:

- `cargo test --lib` — 121 passed (includes 8 generator tests: 4 pre-existing + 4 new)
- `cargo test --test test_generate` — 11 passed (ALL existing tests pass without modification — D-13-18 regression guard)
- `cargo clippy --all-targets --lib -- -D warnings` — exits 0
- `cargo fmt -- --check` — exits 0
- `grep -cE 'Tier::Tier[45] =>' src/generator/mod.rs` — returns `2` (exactly two new match arms)
- `grep -n '/cb/v4/'` and `grep -n '/cb/v5/'` in `src/generator/mod.rs` — both match
- `grep -n 'nonce::derive_seed' src/generator/mod.rs` — matches inside Tier5 arm
- `grep -n 'verification_seed' src/generator/mod.rs` — matches in JSON-LD format string
- `grep -c 'seed_scripts_json' src/generator/mod.rs` — returns `4` (accumulator comment + declaration + join + context! key) ≥ 3
- `grep -c 'seed_scripts_json | safe' assets/templates/index.html.jinja` — returns exactly `1`
- `git diff --stat src/server/mod.rs src/store/mod.rs src/broker/mod.rs src/types.rs src/catalog/mod.rs src/nonce.rs` — empty (Plan 03 touched ONLY src/generator/mod.rs and its template, as declared)
- `git diff assets/templates/index.html.jinja` — purely additive (one `+` line, zero `-` lines)

Plan 04's T5 handler will call `nonce::derive_seed(&url_path_nonce)` (with `.ok()` defensive handling), look up the T5 formula constants by `payload_id` via the nonce_map, compute the expected proof, and compare against the URL-path `{proof}` parameter — all inputs the server needs are now either persisted (nonce_map rows with `payload_id`) or reproducible (seed from nonce). The generator's only contract with the server is: the seed visible in the HTML's JSON-LD block equals `derive_seed(nonce)` for the nonce in the same URL path, which this plan establishes and tests enforce.

## Self-Check: PASSED

Verified:
- FOUND: `src/generator/mod.rs` with `Tier::Tier4 =>`, `Tier::Tier5 =>`, `/cb/v4/`, `/cb/v5/`, `nonce::derive_seed`, `verification_seed`, `seed_scripts_json` (4 occurrences), `t5_seed_scripts`
- FOUND: `assets/templates/index.html.jinja` with exactly one `seed_scripts_json | safe` block
- FOUND: commit `dd4f105` (Task 1 RED — tests only)
- FOUND: commit `df3ad2b` (Task 1 GREEN — implementation + template wiring)
- FOUND: 8 generator unit tests pass (4 pre-existing + 4 new)
- FOUND: 11 `tests/test_generate.rs` integration tests pass unmodified
- FOUND: clippy + fmt clean

---
*Phase: 13-tiers-4-5-backend-payloads-routes-store*
*Completed: 2026-04-24*
