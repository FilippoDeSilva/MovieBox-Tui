# Contributing

## Requirements

- Rust 1.90+ (`rust-version` in Cargo.toml).
- `cargo` with `fmt` and `clippy` components.

## Build

```bash
cargo build            # debug
cargo build --release  # release (lto, strip)
```

## Development loop

```bash
cargo fmt                                  # format
cargo clippy --all-targets -- -D warnings  # lint (must be clean)
cargo build
cargo test --lib
```

The repository's pre-commit hook runs `cargo fmt` and
`cargo clippy --all-targets -- -D warnings`; a commit is rejected if either fails.

## Running

`moviebox-tui` needs an interactive terminal (raw mode + alternate screen). For logs,
see [logging.md](logging.md); `MOVIEBOX_LOG=info` is handy during development.

## Structure

The crate is organized into focused modules; `docs/modules.md` and
`docs/architecture.md` are the map. `tui/app/` holds the application object: `mod.rs`
is the thin `handle_action` dispatcher, and each `handle_*` method lives in its own
module (`requests`, `playback`, `download`, `navigation`, `tv`, `system`, `keyboard`).

## Commit conventions

- Conventional Commits: `feat(scope): …`, `fix(scope): …`, `refactor(scope): …`,
  `docs(scope): …`, `chore(scope): …`.
- Keep changes focused; one logical change per commit.
- Refactors must be behavior-neutral (bodies moved verbatim) and pass the gate.
- When a change surfaces an error, fix it in a follow-up commit rather than amending,
  so the history documents the find and the fix.

## Documentation

- Keep `docs/` up to date with the code. Update the relevant topic doc in the same
  change that alters behavior, and keep `docs/README.md`'s status column accurate.

## CI

`.github/workflows/ci.yml` runs fmt, clippy (`-D warnings`), `cargo audit`, and
`cargo package` on macOS, Linux, and Windows.
