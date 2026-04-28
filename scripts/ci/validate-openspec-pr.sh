#!/usr/bin/env bash
set -euo pipefail

event_path="${1:-${GITHUB_EVENT_PATH:-}}"
event_name="${2:-${GITHUB_EVENT_NAME:-}}"

if [[ "${event_name}" != "pull_request" && "${event_name}" != "pull_request_target" ]]; then
    echo "OpenSpec evidence validation skipped for ${event_name:-non-PR event}."
    exit 0
fi

if [[ -z "${event_path}" || ! -f "${event_path}" ]]; then
    echo "OpenSpec evidence validation failed: missing GitHub pull request event payload."
    exit 1
fi

python3 - "${event_path}" <<'PY'
import json
import os
import re
import subprocess
import sys

event_path = sys.argv[1]

with open(event_path, "r", encoding="utf-8") as event_file:
    event = json.load(event_file)

pull_request = event.get("pull_request") or {}
body = pull_request.get("body") or ""
visible_body = re.sub(r"<!--.*?-->", "", body, flags=re.DOTALL)
base_sha = (pull_request.get("base") or {}).get("sha")
head_sha = (pull_request.get("head") or {}).get("sha")

guarded_patterns = (
    "crates/",
    "src/",
    ".github/workflows/",
    "scripts/ci/",
)
guarded_files = {
    "Cargo.toml",
    "Cargo.lock",
    ".github/PULL_REQUEST_TEMPLATE.md",
    "CONTRIBUTING.md",
    "mise.toml",
    "openspec/config.yaml",
    "rust-toolchain.toml",
}


def changed_files_from_environment():
    raw_files = os.environ.get("OPENSPEC_CHANGED_FILES", "")
    return [line.strip() for line in raw_files.splitlines() if line.strip()]


def changed_files_from_git():
    if not base_sha or not head_sha:
        return []

    try:
        result = subprocess.run(
            ["git", "diff", "--name-only", f"{base_sha}...{head_sha}"],
            check=True,
            capture_output=True,
            text=True,
        )
    except subprocess.CalledProcessError:
        result = subprocess.run(
            ["git", "diff", "--name-only", base_sha, head_sha],
            check=True,
            capture_output=True,
            text=True,
        )

    return [line.strip() for line in result.stdout.splitlines() if line.strip()]


changed_files = changed_files_from_environment() or changed_files_from_git()

if not changed_files:
    print("OpenSpec evidence validation skipped: no changed files detected.")
    sys.exit(0)

guarded_changes = [
    path
    for path in changed_files
    if path in guarded_files or any(path.startswith(pattern) for pattern in guarded_patterns)
]

if not guarded_changes:
    print("OpenSpec evidence validation passed: no guarded paths changed.")
    sys.exit(0)

has_openspec_reference = re.search(r"openspec/(changes|specs)/[^\s)]+", visible_body) is not None
has_no_spec_justification = re.search(
    r"no[- ]openspec[- ]required\s*:\s*\S|no[- ]spec[- ]required\s*:\s*\S",
    visible_body,
    re.IGNORECASE,
) is not None

if has_openspec_reference or has_no_spec_justification:
    print("OpenSpec evidence validation passed.")
    sys.exit(0)

print("OpenSpec evidence validation failed.")
print("Guarded paths changed:")
for path in guarded_changes:
    print(f"- {path}")
print("")
print("Add an OpenSpec reference such as `openspec/changes/<change-name>` or")
print("`openspec/specs/<capability>` to the PR body. For docs-only, typo-only,")
print("formatting-only, dependency-free housekeeping, or other non-behavioral")
print("changes, add `No OpenSpec required: <reason>` instead.")
sys.exit(1)
PY
