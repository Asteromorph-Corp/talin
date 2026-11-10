#!/usr/bin/env bash

# Reports where the API's published description has moved on from this crate's.
#
# Usage: spec/check.sh <provider> [oasdiff arguments...]
#
# <provider> names a directory under spec/, such as `claude_messages`.

set -euo pipefail
cd "$(dirname "$0")/.."

provider="${1:?usage: spec/check.sh <provider> [oasdiff arguments...]}"
shift
dir="spec/$provider"
if [[ ! -f "$dir/oasdiff.yaml" ]]; then
    printf 'check.sh: %s has no oasdiff.yaml; is `%s` a provider?\n' "$dir" "$provider" >&2
    exit 2
fi

oasdiff="${OASDIFF:-oasdiff}"

# Parse `--format` / `-f` flag.
format=text
previous=
for argument in "$@"; do
    case "$argument" in
        --format=*) format="${argument#--format=}" ;;
        *)
            if [[ "$previous" == "--format" || "$previous" == "-f" ]]; then
                format="$argument"
            fi
            ;;
    esac
    previous="$argument"
done

if [[ "$format" == markdown ]]; then
    heading_prefix='##'
    template=(--template spec/oasdiff-changelog.md)
else
    heading_prefix='==>'
    template=()
fi

normalized="$(mktemp -d)"
merged_ignore="$(mktemp)"

cleanup() {
    rm -rf "$normalized" "$merged_ignore"
}
trap cleanup EXIT

# Merge permanent and deferred ignores into a temporary file.
# The merged file overrides oasdiff.yaml's err-ignore and warn-ignore settings.
cat "$dir/oasdiff-ignore.txt" "$dir/oasdiff-ignore-deferred.txt" > "$merged_ignore"

for side in generated upstream; do
    spec/normalize/normalize.sh "$provider" "$dir/openapi.$side.json" > "$normalized/$side.json"
done

status=0

for direction in "generated upstream" "upstream generated"; do
    read -r base revision <<< "$direction"

    printf "%s %s -> %s\n" "$heading_prefix" "$base" "$revision"
    "$oasdiff" --config "$dir/oasdiff.yaml" breaking \
        "$normalized/$base.json" \
        "$normalized/$revision.json" \
        "${template[@]}" \
        --err-ignore "$merged_ignore" \
        --warn-ignore "$merged_ignore" \
        "$@" || status=$?
done

exit "$status"
