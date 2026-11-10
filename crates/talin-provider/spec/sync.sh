#!/usr/bin/env bash

# Refreshes the committed copy of the API's published description.
#
# Usage: spec/sync.sh <provider> [url]
#
# Reads where to fetch from in spec/<provider>/upstream.json:
#
# - `url`: where the API publishes its description.
# - `path`: the path the description must describe, to tell it from an error page.
# - `convert` (optional): a script, relative to spec/<provider>/, that converts what `url` serves
#   into an OpenAPI description. For an API that publishes something other than OpenAPI.
# - `reference` (optional): where, relative to spec/<provider>/, to keep what `url` served before
#   it was converted.
#
# Takes a URL to fetch from instead, which is how the `Spec sync` workflow is exercised without
# waiting for a drift.

set -euo pipefail
cd "$(dirname "$0")/.."

provider="${1:?usage: spec/sync.sh <provider> [url]}"
dir="spec/$provider"
config="$dir/upstream.json"
if [[ ! -f "$config" ]]; then
    printf 'sync.sh: %s has no upstream.json; is `%s` a provider?\n' "$dir" "$provider" >&2
    exit 2
fi

url="${2:-$(jq -r '.url' "$config")}"
path="$(jq -r '.path' "$config")"
convert="$(jq -r '.convert // empty' "$config")"
reference="$(jq -r '.reference // empty' "$config")"

raw="$(mktemp)"
json="$(mktemp)"
trap 'rm -f "$raw" "$json"' EXIT

curl --fail --silent --show-error --location --output "$raw" "$url"

if [[ -n "$convert" ]]; then
    "$dir/$convert" "$raw" > "$json"
else
    cp "$raw" "$json"
fi

# An error page served with a 200 is worth ruling out before anything lands. Once converted, it
# would be an empty description rather than a failure, so this checks what came out.
if ! jq -e --arg path "$path" 'has("openapi") and (.paths | has($path))' "$json" > /dev/null 2>&1; then
    {
        printf '%s\n' "sync.sh: what $url served is not a description of $path."
        printf 'Fetched %s bytes, which read as %s operations and %s schemas.\n' \
            "$(wc -c < "$raw")" \
            "$(jq '(.paths // {}) | length' "$json" 2>/dev/null || echo 0)" \
            "$(jq '(.components.schemas // {}) | length' "$json" 2>/dev/null || echo 0)"
        printf 'The first lines of what was fetched:\n'
        head -n 15 "$raw" | sed 's/^/  /'
    } >&2
    exit 1
fi

# Both files are written together, so the committed reference stays the one the committed
# description was made from. Reading either on its own says what the API published; reading them
# side by side says what the converter made of it.
# Copied with a redirect rather than `cp`, which would carry the temporary file's mode over.
if [[ -n "$reference" ]]; then
    cat "$raw" > "$dir/$reference"
fi

# Sorted, so a re-serialization upstream is not a diff here. Key order carries no meaning in
# OpenAPI; arrays keep the order they were published in.
jq -S . "$json" > "$dir/openapi.upstream.json"
