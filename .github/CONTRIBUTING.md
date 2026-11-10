# Contributing to Talin

Thanks for your interest in contributing! This document covers how to report issues, propose
changes, and get a pull request merged.

## Code of Conduct

This project follows the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct).
By participating, you are expected to uphold it. If you have concerns about someone's conduct,
please [reach out to us](mailto:comm@asteromorph.com).

## Reporting Issues

Use the [issue tracker](https://github.com/Asteromorph-Corp/talin/issues) for bug reports and
feature requests, using the provided templates.

For a bug report, include the version you're on, the steps to reproduce, and what you expected to
happen instead. For a feature request, describe the problem you're running into before proposing a
solution.

## Development Setup

You'll need:

- [Rust](https://www.rust-lang.org/tools/install), at least the MSRV declared in
  [`Cargo.toml`](Cargo.toml) (`workspace.package.rust-version`)
- The pinned nightly toolchain in [`.github/nightly-toolchain`](.github/nightly-toolchain),
  for `rustfmt`'s unstable options

Clone the repository and build the workspace:

```bash
git clone https://github.com/Asteromorph-Corp/talin
cd talin
cargo build --workspace
```

## Branches and Commits

- All pull requests should be made against the `main` branch. Even when you wish to have your change 
  applied in the previous versions of the crate, please make your PR against `main`. The maintainers
  will backport the change to the appropriate release branch(es) if necessary.
- A release PR is automatically opened on every merge to `main`. A new version is published once the
  release PR is merged.
- Only maintainers can push branches to this repo directly, so if you're an outside contributor,
  work from a fork and name your branch however you like — the PR title is what matters (see
  [Pull Requests](#pull-requests)). If you do have push access, name your branch
  `<type>/<kebab-case-description>` (e.g. `feat/streams`, `fix/http-auth-failure`), where
  `<type>` is one of the commit types below.
- Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/),
  in English:

    ```text
    <type>(<scope>): <message>

    <body>
    ```

    `(<scope>)` and `<body>` are optional. `<message>` is a short, lowercase, imperative summary with
    no trailing period (e.g. `feat: add optional authentication in request`).

    | `<type>`   | Use it for                                       |
    | :--------- | :----------------------------------------------- |
    | `feat`     | A new feature or capability                      |
    | `fix`      | A bug fix                                        |
    | `docs`     | Documentation or comment changes only            |
    | `style`    | Formatting/style changes with no behavior change |
    | `refactor` | Code restructuring with no behavior change       |
    | `test`     | Adding or updating tests                         |
    | `chore`    | Build, tooling, dependency, or release changes   |
    | `etc`      | Anything that doesn't fit the above              |

    Mark a breaking change with a `!` before the colon (`feat(api)!: change response format`) and/or
    a `BREAKING CHANGE: <description>` line in the body.

- For a branch that is meant to be squashed into `main`, intermediate commits within the branch do not
  need to follow this convention strictly. You should, however, keep commits atomic.

## Pull Requests

- Open your PR as a draft while you're still implementing. Only mark it "Ready for review"
  once CI is green.
- Fill in the [PR template](.github/PULL_REQUEST_TEMPLATE.md): what the PR closes (if anything),
  a summary, a changelog-worthy list of changes, the motivation, (if non-obvious) the
  implementation approach, and how you tested it.
- Title the PR the way you'd write a commit message for the whole change
  (e.g. `feat: add optional authentication in request`). A maintainer might rename your PR to match
  this convention.
- A maintainer will review and squash-merge once CI passes and the PR is approved. As most PRs
  go through at least one round of feedback, please be patient and responsive to review comments.

## Test suite

Our CI runs the following on every push and PR. We suggest running these before opening a PR, since
iterating on your local development settings would be easier than waiting for the CI completion.

```bash
# Formatting
cargo +$(cat .github/nightly-toolchain) fmt --all -- --check

# Lints
cargo clippy --workspace --all-targets -- -D warnings

# Tests
cargo test --workspace --all-targets
cargo test --workspace --doc

# Docs
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --document-private-items
```

CI additionally checks the MSRV and a minimal-versions resolution of dependencies. As these are
harder to reproduce locally, don't worry if you can't run them yourself.

## Versioning

Talin follows [Semantic Versioning](https://semver.org/). Since the crate is still pre-1.0,
we use the following convention for what bumps `MINOR` versus `PATCH`:

- `MINOR`: a breaking change to the public interface (API, CLI, configuration/input formats), or
  the addition of a major feature (something that meaningfully changes how the crate is used, as
  opposed to a QoL improvement or a new option).
- `PATCH`: everything else, bug fixes and non-breaking improvements.

If your change breaks the public interface, please call that out explicitly in your PR title,
description, and commit message (see [Branches and Commits](#branches-and-commits)).
