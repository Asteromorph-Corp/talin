#!/usr/bin/env bash

# Validates that no deferred ignore entries have expired.
#
# Usage: spec/check-deferred.sh [--crate-version <version>]
#
# If --crate-version is not provided, reads from Cargo.toml.
# Exits 0 if all deferrals are still valid, 1 if any have expired.

set -euo pipefail
cd "$(dirname "$0")/.."

crate_version=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --crate-version)
            crate_version="$2"
            shift 2
            ;;
        *)
            echo "Unknown argument: $1" >&2
            echo "Usage: spec/check-deferred.sh [--crate-version <version>]" >&2
            exit 2
            ;;
    esac
done

# Get version from Cargo.toml if not provided
if [[ -z "$crate_version" ]]; then
    # The crate uses workspace version, so check root Cargo.toml
    crate_version=$(grep -E '^version\s*=' ../../Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
    if [[ -z "$crate_version" ]]; then
        echo "Could not determine crate version from Cargo.toml" >&2
        exit 2
    fi
fi

echo "Checking deferred ignores against crate version: $crate_version"

# Parse semver components using parameter expansion (avoids IFS issues in loops)
parse_version() {
    local version="$1" prefix="$2"
    local major minor patch tmp
    major="${version%%.*}"
    tmp="${version#*.}"
    minor="${tmp%%.*}"
    patch="${tmp#*.}"
    patch="${patch%%-*}"  # Strip pre-release suffix
    printf -v "${prefix}_major" '%s' "$major"
    printf -v "${prefix}_minor" '%s' "$minor"
    printf -v "${prefix}_patch" '%s' "$patch"
}

parse_version "$crate_version" current

expired_count=0
expired_entries=""

# Check each provider's deferred file
for deferred_file in spec/*/oasdiff-ignore-deferred.txt; do
    [[ -f "$deferred_file" ]] || continue

    provider=$(dirname "$deferred_file" | xargs basename)

    # Parse metadata lines: #@ <version> <action>
    while IFS= read -r line; do
        if [[ "$line" =~ ^#@[[:space:]]+([0-9]+\.[0-9]+\.[0-9]+)([[:space:]]+(.*))?$ ]]; then
            until_version="${BASH_REMATCH[1]}"
            action="${BASH_REMATCH[3]}"
            parse_version "$until_version" until

            # Compare versions: expired if current >= until
            expired=false
            if (( current_major > until_major )); then
                expired=true
            elif (( current_major == until_major )); then
                if (( current_minor > until_minor )); then
                    expired=true
                elif (( current_minor == until_minor && current_patch >= until_patch )); then
                    expired=true
                fi
            fi

            if $expired; then
                ((++expired_count))
                expired_entries+="
  - \`$provider\`: ${action:-<no action specified>}
    Expired at: $until_version (current: $crate_version)"
            fi
        fi
    done < "$deferred_file"
done

if (( expired_count > 0 )); then
    cat >&2 <<EOF
ERROR: Found $expired_count expired deferred ignore(s).

These deferrals were set to expire at or before version $crate_version.
They must be addressed before releasing this version.
$expired_entries

To fix:
1. Implement the deferred changes (update types, add fields, etc.)
2. Remove the expired entries from oasdiff-ignore-deferred.txt
EOF
    exit 1
fi

echo "All deferred ignores are still valid."
exit 0
