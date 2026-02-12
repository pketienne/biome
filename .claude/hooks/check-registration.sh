#!/usr/bin/env bash
# Verify all 3 registration systems are wired for each language analyzer crate.
#
# Usage: check-registration.sh [language]
#   If language is provided, checks only that language.
#   If omitted, checks all biome_*_analyze crates.
#
# Exit codes:
#   0 — all registrations found
#   1 — missing registrations (details on stderr)

set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || echo '.')"
MACROS_DIR="$REPO_ROOT/crates/biome_configuration_macros"
CODEGEN_DIR="$REPO_ROOT/xtask/codegen"

errors=0

check_language() {
    local lang="$1"
    local crate="biome_${lang}_analyze"

    # System 1: xtask/codegen analyzer knows about the crate
    if ! grep -rq "$crate" "$CODEGEN_DIR/src/" 2>/dev/null; then
        echo "MISSING: System 1 (xtask/codegen analyzer) does not reference $crate" >&2
        errors=$((errors + 1))
    fi

    # System 2: configuration codegen includes the language
    # Check generated rules or codegen source for the language
    if ! grep -rq "$lang" "$CODEGEN_DIR/src/" 2>/dev/null && \
       ! grep -rq "$crate" "$REPO_ROOT/crates/biome_configuration/src/" 2>/dev/null; then
        echo "MISSING: System 2 (configuration codegen) does not reference $lang" >&2
        errors=$((errors + 1))
    fi

    # System 3: proc macro calls visit_registry (the silent failure case)
    if [ -d "$MACROS_DIR" ]; then
        if ! grep -rq "${crate}::visit_registry\|${crate}" "$MACROS_DIR/src/" 2>/dev/null; then
            echo "MISSING: System 3 (biome_configuration_macros) does not call ${crate}::visit_registry" >&2
            echo "  This causes SILENT FAILURE: rules compile and pass unit tests but never fire at runtime." >&2
            errors=$((errors + 1))
        fi
    fi
}

if [ $# -ge 1 ]; then
    # Check specific language
    check_language "$1"
else
    # Check all analyzer crates
    for crate_dir in "$REPO_ROOT"/crates/biome_*_analyze; do
        if [ -d "$crate_dir" ]; then
            lang=$(basename "$crate_dir" | sed 's/^biome_//;s/_analyze$//')
            check_language "$lang"
        fi
    done
fi

if [ $errors -gt 0 ]; then
    echo "" >&2
    echo "Found $errors missing registration(s). See references/registration-systems.md for details." >&2
    exit 1
fi

echo "All registration systems verified."
exit 0
