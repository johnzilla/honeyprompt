#!/usr/bin/env python3
"""
Fire one sample callback per tier against a running honeyprompt serve instance.

Usage:
    python3 fire_callbacks.py            # defaults to http://127.0.0.1:8080
    python3 fire_callbacks.py http://127.0.0.1:8090

Prereqs:
    ./target/release/honeyprompt generate   # creates output/callback-map.json
    ./target/release/honeyprompt serve      # running on the same HOST:PORT

This is a Phase 14 Task 4 smoke helper — intentionally simple, not a test-agent replacement.
"""

import base64
import json
import subprocess
import sys

HOST = sys.argv[1] if len(sys.argv) > 1 else "http://127.0.0.1:8080"

# Pick one nonce per tier from the generated callback map.
with open("output/callback-map.json") as f:
    mappings = json.load(f)

by_tier = {}
for m in mappings:
    by_tier.setdefault(m["tier"], m)  # first-seen wins per tier

# Hard-coded T5 formula constants from assets/catalog/tier5.toml.
T5_FORMULAS = {
    "t5-semantic-prose":    (42, 17, 1000),
    "t5-html-comment":      (99, 31, 1000),
    "t5-invisible-element": (7,  97, 1000),
}


def fire(url):
    print(f"  -> {url}")
    subprocess.run(
        ["curl", "-sS", "-o", "/dev/null", "-w", "     HTTP %{http_code}\n", url],
        check=False,
    )


print(f"Firing against {HOST}\n")

# T1, T2, T3 — simple GETs (catalog routes all three to /cb/v1/ by design)
for tier in ("Tier1", "Tier2", "Tier3"):
    nonce = by_tier[tier]["nonce"]
    print(f"{tier}:")
    fire(f"{HOST}/cb/v1/{nonce}")

# T4 — base64(sorted tool list), URL-safe, no padding
t4 = by_tier["Tier4"]
tools = b"browse_page,code_execution,web_search"
b64 = base64.urlsafe_b64encode(tools).rstrip(b"=").decode()
print("Tier4:")
fire(f"{HOST}/cb/v4/{t4['nonce']}/{b64}")

# T5 — compute proof = ((seed + A) * B) % MOD, zero-pad to 3 digits
t5 = by_tier["Tier5"]
nonce = t5["nonce"]
seed = int(nonce[:8], 16)
A, B, MOD = T5_FORMULAS[t5["payload_id"]]
proof = f"{((seed + A) * B) % MOD:03d}"
print(f"Tier5 ({t5['payload_id']}, seed={seed}, proof={proof}):")
fire(f"{HOST}/cb/v5/{nonce}/{proof}")

print("\nDone. Check the Monitor TUI — you should see 5 rows (T1..T5) with evidence cells filled.")
