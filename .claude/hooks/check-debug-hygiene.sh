#!/usr/bin/env bash
# Check for debug artifacts in staged or modified Rust files.
#
# Usage: check-debug-hygiene.sh [file...]
#   If files provided, checks those files.
#   If omitted, checks all modified .rs files (staged + unstaged).
#
# Exit codes:
#   0 — no debug artifacts found
#   1 — debug artifacts found (details on stdout)

set -euo pipefail

artifacts_found=0

check_file() {
    local file="$1"
    [ -f "$file" ] || return 0
    [[ "$file" == *.rs ]] || return 0

    # Skip test files — debug macros are acceptable in tests
    if [[ "$file" == */tests/* ]] || [[ "$file" == */test_* ]]; then
        return 0
    fi

    local hits
    hits=$(grep -n 'dbg!\|eprintln!\|println!' "$file" 2>/dev/null || true)
    if [ -n "$hits" ]; then
        echo "DEBUG ARTIFACTS in $file:" >&2
        echo "$hits" >&2
        echo "" >&2
        artifacts_found=$((artifacts_found + 1))
    fi
}

if [ $# -ge 1 ]; then
    # Check specific files
    for f in "$@"; do
        check_file "$f"
    done
else
    # Check all modified .rs files
    REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || echo '.')"
    while IFS= read -r file; do
        check_file "$REPO_ROOT/$file"
    done < <(git diff --name-only --diff-filter=ACMR HEAD 2>/dev/null || true)
    while IFS= read -r file; do
        check_file "$REPO_ROOT/$file"
    done < <(git diff --name-only --cached --diff-filter=ACMR 2>/dev/null || true)
fi

if [ $artifacts_found -gt 0 ]; then
    echo "Found debug artifacts in $artifacts_found file(s). Remove before committing." >&2
    exit 1
fi

exit 0
