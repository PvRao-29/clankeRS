#!/usr/bin/env bash
#
# Open the initial clankeRS issue set on GitHub.
#
# Prerequisite:  gh auth login
# Usage:         bash scripts/open_issues.sh
#
# See docs/ISSUES.md for the full detail and current status of each item.

set -euo pipefail

REPO="${CLANKERS_REPO:-PvRao-29/clankeRS}"

if ! command -v gh >/dev/null 2>&1; then
  echo "error: GitHub CLI (gh) is not installed. See https://cli.github.com/" >&2
  exit 1
fi

if ! gh auth status >/dev/null 2>&1; then
  echo "error: not logged in. Run 'gh auth login' first." >&2
  exit 1
fi

create() {
  local title="$1" body="$2"
  echo "Opening: $title"
  gh issue create --repo "$REPO" --title "$title" --body "$body"
}

create "[cli] Add \"clankers doctor\" status/environment command" \
"Print the README Status table and check the local environment: Rust toolchain, ONNX Runtime availability, and presence of sample_data/models/*.onnx + their expected_output.json references.

Acceptance: exits non-zero with a clear message when the golden path cannot run."

create "[ml] Live PyTorch-vs-ONNX validation (--pytorch / --checkpoint)" \
"validate-model already compares Rust ONNX output against a stored PyTorch reference (expected_output.json) with a real, non-zero error and honest pass/fail.

Remaining: honor --pytorch scripts/model.py:Class and --checkpoint weights.pt to run PyTorch live and generate the reference on the fly (torch-optional, behind a flag). Surface 'PyTorch latency p50' on the live path."

create "[data] Make MCAP replay deterministic (simulated clock)" \
"Replay measures per-message latency against wall-clock Instant. Add a simulated-clock mode so replay summaries (dropped messages, deadline misses) are reproducible in CI regardless of host speed.

Acceptance: two runs on the same log produce identical summaries."

create "[demo] Harden camera_perception_node as the golden-path demo" \
"Done: 'clankers demo camera-perception' validates the model, replays a real MCAP, and reports measured latency.

Follow-ups: report release-mode latency, decode real image frames from the log for inference instead of a fixed input, and add a visualization hook."

create "[docs] Fresh-clone quickstart with expected output" \
"Done: README has a verified quickstart, Status table, and real expected output.

Remaining: align docs/getting_started.md, docs/model_validation.md, and docs/mcap_replay.md with the exact verified commands and outputs."

create "[ros2] Add real rclrs backend behind a ros2 feature flag" \
"Add a 'ros2' feature to clankers-ros2 that swaps the in-memory SimBus for real rclrs publishers/subscribers. Keep the simulated bus as default so --features ros2 is opt-in and no ROS install is needed for dev."

create "[repo] Add .gitattributes to fix GitHub language stats" \
"Done: .gitattributes marks sample data / models / vendored trees as linguist-vendored.

Verify the language bar reads as Rust after the next push; if C++/Makefile persist, find and remove committed build artifacts (they should not be tracked)."

create "[ci] Add golden-path CI workflow" \
"Done: .github/workflows/ci.yml runs fmt + clippy(-D warnings) + test, then a golden-path job (inspect, validate both models, demo, inference node).

Extend: OS/Rust matrix, release-build latency check, and caching of the ONNX Runtime download."

echo "Done. Opened the initial issue set on $REPO."
