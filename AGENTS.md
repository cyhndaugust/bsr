# Repository Guidelines

## Project Structure & Module Organization
- `src/main.rs` is the CLI entry point; it delegates to the library crate (`bsr::handles::run`).
- `src/lib.rs` re-exports internal modules: `args`, `handles`, `types`, and `utils`.
- `src/args.rs` defines CLI argument parsing (via `clap`).
- `src/handles.rs` contains the main command flow and user interaction (via `inquire`).
- `src/types.rs` and `src/utils.rs` hold shared data structures and helper functions.
- `target/` is build output and should not be edited manually.

## Build, Test, and Development Commands
- `cargo build` — compile the project (debug build).
- `cargo run -- <args>` — run the CLI locally with arguments.
- `cargo build --release` — optimized build for distribution.
- `cargo test` — run tests (none are currently present).
- `cargo fmt` / `cargo clippy` — format and lint if those tools are installed.

## Coding Style & Naming Conventions
- Follow standard Rust style: 4-space indentation, `snake_case` for functions/modules, `CamelCase` for types.
- Keep module boundaries clear: CLI parsing in `args`, runtime flow in `handles`, shared helpers in `utils`.
- Prefer `anyhow::Result` for error propagation in application code.

## Testing Guidelines
- No tests exist yet. Add unit tests in `src/<module>.rs` under `mod tests { ... }` or integration tests in `tests/`.
- Name tests with descriptive `snake_case` function names (e.g., `parses_dir_path`).

## Commit & Pull Request Guidelines
- Commit history shows short, lowercase summaries (e.g., “add dir”, “parse a dir path”). Keep messages brief and imperative.
- PRs should include: a clear description, rationale for behavior changes, and example usage (CLI commands or screenshots of output) when applicable.

## Configuration & Security Notes
- This is a local CLI tool; avoid writing secrets to disk or logs.
- When adding configuration, prefer CLI flags first and document them in `src/args.rs`.
