#!/usr/bin/env bash

# Rewrites an OpenAPI description into the shape `check.sh` compares.
#
# Usage: spec/normalize/normalize.sh <provider> <description>

set -euo pipefail

provider="${1:?usage: spec/normalize/normalize.sh <provider> <description>}"

# Copied before the `cd` below, so that the argument resolves against the caller's directory.
spec="$(mktemp)"
trap 'rm -f "$spec" "$spec.step"' EXIT
cp "${2:?usage: spec/normalize/normalize.sh <provider> <description>}" "$spec"

cd "$(dirname "$0")/../.."

oasdiff="${OASDIFF:-oasdiff}"

# Rewrites `$spec` with the output of the command it is given.
apply() {
    "$@" < "$spec" > "$spec.step"
    mv "$spec.step" "$spec"
}

# Wraps a `$ref` arm of a `oneOf` union inside an `allOf`.
apply jq -f spec/normalize/wrap-ref-arm.jq
# Titles an arm of a `oneOf` union with the name of the component it references.
apply jq -f spec/normalize/title-arm-by-ref.jq
# Merges each `allOf` into the object it describes.
apply "$oasdiff" flatten /dev/stdin
# Collapses a single-armed `oneOf` union to its content.
apply jq -f spec/normalize/collapse-single-union.jq
# Titles an arm of a `oneOf` union with the value of the tag property in that arm.
apply jq -f spec/normalize/title-arm-by-tag.jq

cat "$spec"
