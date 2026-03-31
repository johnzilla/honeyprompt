# Deferred Items — Phase 05

## Out-of-Scope Issues Discovered During 05-03

These issues were discovered during 05-03 execution but exist in files not modified by this plan.
They are logged here for tracking and are NOT fixed in 05-03.

### clippy -D warnings failures (pre-existing)

**src/store/mod.rs**
- `too_many_arguments (9/7)` — `insert_callback_event` at line 54

**src/monitor/mod.rs**
- `map_or can be simplified` at line 473
- `very complex type` at line 735
- `unnecessary closure to substitute Result::Err` at line 763
- `assert_eq! with literal bool` at line 1035 (test)

### cargo fmt failures (pre-existing, widespread)

The following files have unformatted code that `cargo fmt --all -- --check` would reject:
- src/broker/mod.rs
- src/catalog/mod.rs
- src/cli/mod.rs
- src/crawler_catalog/mod.rs
- src/fingerprint/mod.rs
- src/generator/mod.rs
- src/monitor/mod.rs
- src/report/mod.rs
- src/server/mod.rs
- src/store/mod.rs
- src/types/mod.rs

Recommendation: Run `cargo fmt --all` once in a dedicated formatting commit before enabling
`cargo fmt --all -- --check` in CI.
