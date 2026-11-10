# Spec drift

This crate describes wire formats it does not own.
These files are how it notices when an API's description of its format and this crate's have parted.

`check.sh` and `sync.sh` are the two entry points; everything else is a stage one of them runs, or a file one of them reads or writes.
Both take the provider to work on, which names a directory under `spec/`:

```sh
spec/check.sh claude_messages
spec/sync.sh gemini_interactions
```

## Layout

Shared across providers:

| Path | Purpose |
| --- | --- |
| `check.sh <provider>` | Compares the provider's two descriptions |
| `check-deferred.sh` | Validates no deferred ignores have expired |
| `sync.sh <provider> [url]` | Fetches the provider's published description |
| `normalize/normalize.sh <provider> <description>` | Rewrites a description into the shape `check.sh` compares |
| `normalize/*.jq` | The rewrites `normalize.sh` applies |
| `oasdiff-changelog.md` | Template for the Markdown reports `check.sh` writes |

Per provider, under `<provider>/`:

| Path | Purpose | Managed by |
| --- | --- | --- |
| `openapi.generated.json` | What this crate's types describe | The `generated_spec_matches_types` unit test |
| `openapi.upstream.json` | What the API publishes | The `Spec sync` workflow |
| `schemas.json` | The enumerations `normalize.sh` and `src/<provider>/spec.rs` normalize | — |
| `upstream.json` | Where and how `sync.sh` fetches the published description | — |
| `oasdiff.yaml` | Configuration file for `oasdiff` | — |
| `oasdiff-ignore.txt` | Permanent reports `check.sh` ignores | — |
| `oasdiff-ignore-deferred.txt` | Temporary ignores with expiration metadata | — |

Claude also keeps:

| Path | Purpose | Managed by |
| --- | --- | --- |
| `claude_messages/reference.upstream.md` | The page `openapi.upstream.json` was converted from | The `Spec sync` workflow |
| `claude_messages/from-markdown.py` | Converts that page into an OpenAPI description | — |

## Spec generation

Running the `generated_spec_matches_types` unit tests with `SPEC_BLESS=1` regenerates every `openapi.generated.json` from the types written in the crate.

```sh
SPEC_BLESS=1 cargo test -p talin-provider generated_spec_matches_types
```

## Drift detection

The `Spec` workflow runs `check.sh` for each provider to compare its two descriptions and report any drift.
It runs on every push to `main` and on pull requests, and fails if it finds any drift.

Running it locally needs `jq` and the `oasdiff` release `.github/actions/oasdiff` pins, either on `PATH` or named by `OASDIFF`.

## Upstream spec sync

The `Spec sync` workflow runs `sync.sh` for each provider weekly and, when a provider's published description has changed, opens a pull request against that provider's `*.upstream.*` files, one pull request and branch (`chore/spec-sync/<provider>`) per provider.
The `drift` check runs on that pull request and detects if the Rust source requires an update.
One must push commits to the open pull request to resolve any drift and only merge after the check passes.
While a provider's sync is open, later runs stand down for that provider rather than rebuild its branch.

`upstream.json` tells `sync.sh`:

- `url`: where the API publishes its description.
- `path`: the path the description must describe, to tell it from an error page.
- `convert` (optional): a script, relative to the provider directory, that converts what `url` serves into an OpenAPI description.
- `reference` (optional): where, relative to the provider directory, to keep what `url` served before it was converted.

Anthropic does not publish an OpenAPI description of the API.
However, they do publish their [API reference](https://platform.claude.com/docs/en/api/messages.md) as Markdown, which is regular enough that `claude_messages/from-markdown.py` generates an OpenAPI description from it.

The workflow can also be run by hand, for one provider or all of them.
Given an `upstream-url`, it fetches the one provider's description from there instead, which is how the workflow is exercised without waiting for a drift.

It is worth noting that the workflow requires the `RUST_CICD_APP_ID` and `RUST_CICD_APP_PRIVATE_KEY` secrets to be set in order to open a pull request.
Without them, the workflow will fail where it would have opened the pull request.

## Ignoring a report

A provider's `oasdiff-ignore.txt` is a list of lines to ignore in its report.
`oasdiff` matches an ignore line by method, path and message in full, so a line covers exactly one transition.
This prevents a genuine drift from being ignored by accident, and makes it easy to see what each ignore entry is for.
This is also why `.github/actions/oasdiff` pins an exact release.
A version that rewords message stops ignoring it.
Bumping the pin is a change that may need every ignore file rewritten alongside it.

Generally, it is recommended to fix the types or normalization logic to match the API's description, rather than adding an ignore line. The ignore file is for cases where the crate's types are intentionally different from the API's description.
Each ignore file should be kept as small as possible.

The ignore files currently include:

- `claude_messages`: Models the API offers that this crate has taken no `KnownModel` variant for. This includes 4.5 models referenced by their release date. Compaction beta request fields, content blocks, stream deltas, stop reasons and iteration usage are also absent from the standard Messages description fetched by `upstream.json`.
- `gemini_interactions`: Models the API offers that this crate has taken no `KnownModel` variant for, including preview models and gemma models; and agent configuration that this crate does not model.
- `xai_responses`: Nothing.

## Deferring a breaking change

When the upstream API introduces a breaking change that would require a breaking update to this crate, but the change is minor enough that it can wait until the next planned breaking release, you can defer it using `oasdiff-ignore-deferred.txt`.

```txt
# Deferred breaking changes.
#
# Each entry block starts with a metadata line (#@) followed by the ignore lines.
#
# Metadata format:
#   #@ <version> <action>
#
# - version: The version where this deferral expires (must be addressed before releasing).
# - action: (Optional) A short description of the action to take.
#
# Example:
#@ 0.5.0 add `new_field` to response
post /v1/messages added the required property `new_field` to the response with the `200` status
```

`check-deferred.sh` fails CI when the crate version reaches or exceeds the specified version.
