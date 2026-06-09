# Copilot Instructions for linz_S3

## Project Context
- This is a Rust CLI and library for searching LINZ public S3 STAC datasets and optionally downloading tiles.
- Main crate name and binary: `linz_s3`.
- Key modules live under `src/` and `src/linz_s3_filter/`.

## Code Change Guidelines
- Prefer small, focused edits that preserve current behavior unless the task asks for behavior changes.
- Keep public CLI behavior in sync with clap definitions in `src/args.rs` and runtime flow in `src/main.rs`.
- Reuse existing error/reporting patterns from `src/error.rs` instead of introducing new ad-hoc error styles.
- Do not modify generated or build-output content under `target/`.
- Add or update tests in `tests/` when behavior changes.

## Build and Validation
- Build with: `cargo build` (or `cargo build --release` for release artifacts).
- Run tests with: `cargo test`.
- Prefer targeted test runs when possible during iteration (for example, `cargo test test_name`).
- Before finalizing substantial Rust changes, run:
  - `cargo fmt`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test`

## Repository Conventions
- Keep CLI docs/help text accurate and user-focused.
- Prefer explicit, readable naming over compact but unclear code.
- Avoid introducing unnecessary dependencies; use std or existing crate dependencies when practical.
- Preserve existing module organization unless a refactor is explicitly requested.

## CI and Release Notes
- Release automation is defined in `.github/workflows/release.yml`.
- When changing packaging, binary names, or target behavior, ensure release workflow inputs still match the crate/binary outputs.
