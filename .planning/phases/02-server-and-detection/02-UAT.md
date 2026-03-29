---
status: complete
phase: 02-server-and-detection
source: [02-01-SUMMARY.md, 02-02-SUMMARY.md, 02-03-SUMMARY.md]
started: 2026-03-29T04:00:00Z
updated: 2026-03-29T04:15:00Z
---

## Current Test

[testing complete]

## Tests

### 1. Cold Start Smoke Test
expected: Build the project fresh, init and generate a honeypot, then run `honeyprompt serve test-site`. Server boots without errors and prints ready message with bind address, nonce count, and DB path.
result: pass

### 2. Static Honeypot Page Served
expected: While the server is running, curl http://127.0.0.1:8080/. Returns the generated honeypot HTML page with visible human warnings.
result: pass

### 3. Valid Callback Returns 204
expected: Hit /cb/v1/<valid-nonce> with curl. Response status is 204 No Content with an empty body.
result: pass

### 4. Unknown Nonce Returns 204
expected: Hit /cb/v1/totally-fake-nonce with curl. Still returns 204 No Content. Server never reveals nonce validity.
result: pass

### 5. Callback Event Logged to Stdout
expected: After hitting a valid nonce callback URL, the terminal prints a log line showing tier, classification, IP, and user-agent.
result: pass

### 6. JSON Output Mode
expected: Restart with --json flag. Hit a callback URL. Stdout log is structured JSON with fields like tier, session_id, classification.
result: pass

### 7. Crawler Classification
expected: Hit callback with GPTBot/1.0 UA. Log shows KnownCrawler:OpenAI instead of Unknown.
result: pass

### 8. Graceful Shutdown with Summary
expected: Ctrl+C shuts down cleanly and prints detection count summary.
result: pass

## Summary

total: 8
passed: 8
issues: 0
pending: 0
skipped: 0
blocked: 0

## Gaps

[none]
