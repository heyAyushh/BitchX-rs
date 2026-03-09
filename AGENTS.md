# AGENTS.md

## Project overview

BitchY is a terminal-based IRC (Internet Relay Chat) client inspired by the original BitchX. This repository now tracks the Rust rewrite.

## Cursor Cloud specific instructions

### Installed cursor rules and skills

The `.cursor/` directory contains rules and skills for AI-assisted development:

| Source | What it provides | Location |
|--------|-----------------|----------|
| **heyAyushh/stacc** | Karpathy guidelines, clean-code rules, commit/PR format, Rust & TypeScript stack rules, various skills | `.cursor/rules/*.mdc`, `.cursor/skills/` |
| **tyrchen/cursor-rust-rules** | Modular Rust rules (core, quality, features, simple/complex project patterns) | `.cursor/rules/rust/` |
| **awesome-cursor-rules-mdc** | Community Rust best practices | `.cursor/rules/awesome-rust.mdc` |

Key skills in `.cursor/skills/`:
- `karpathy-guidelines/` -- behavioral guidelines to reduce common LLM coding mistakes
- `rust/` -- Rust stack skill
- `typescript/` -- TypeScript stack skill
- `bash-expert/`, `agent-browser/`, `changelog-generator/`, `find-skills/`, `skill-creator/`, `mcp-builder/`, `frontend-design/`

### Rust rewrite

When the Rust rewrite is underway:
- Build: `cargo build`
- Test: `cargo test`
- Lint: `cargo clippy`
- Format: `cargo fmt --check`
- All four must pass before code is considered complete.
- See `.cursor/rules/rust/main.mdc` for the full rule loading system (core, quality, features).

### Rust module structure

The Rust rewrite lives under `src/`. Modules: `irc/` (connection, protocol, commands), `ui/` (TUI app, widgets, input), `plugin/` (C ABI plugin loader + macro helper), `scripting/` (alias engine), and `config`. The plugin system uses `libloading` for dynamic `.so` loading with a C ABI contract (`bitchy_plugin_*` symbols). Plugin commands: `/loaddll`, `/unloaddll`, `/listdll`.

### Crypto crate API notes

- `cbc` 0.1.2 with `cipher` 0.4.4: use `encrypt_padded_mut`/`decrypt_padded_mut` (not `*_vec_mut` variants). Import `BlockEncryptMut`/`BlockDecryptMut` from `cbc::cipher`.
- Blowfish minimum key length is 4 bytes.
- `aes-gcm` 0.10: use `Aes256Gcm::generate_nonce(&mut OsRng)` for nonce generation.

### Workspace and plugin crates

The root `Cargo.toml` defines a workspace with members `"."`, `"plugins/hello"`, and `"plugins/greet"`. Plugin crates are `cdylib` and produce `.so` files in `target/debug/`. Use `cargo build --workspace` and `cargo test --workspace` to include them. Clippy/fmt on the plugin crates: `cargo clippy -p bitchy-plugin-hello -p bitchy-plugin-greet` and `cargo fmt --check -p bitchy-plugin-hello -p bitchy-plugin-greet`. Note: `cargo clippy --workspace` may report pre-existing issues in the main `bitchy` crate; use `-p` to check individual crates.

### Testing caveats

- All 289 tests pass reliably (256 lib + 15 bin + 9 hello plugin + 9 greet plugin), including `irc::client` async tests with mock TCP servers.
- `cargo fmt --check` must pass. `cargo clippy` must produce zero warnings/errors for new code.
- `aws-lc-sys` (transitive dep via `rustls`) requires `cmake` and a C compiler at build time. These are pre-installed in the VM.

### Key directories

| Directory | Purpose |
|-----------|---------|
| `src/` | Rust source code (IRC core, UI, plugin loader, scripting) |
| `bitchx-docs/` | Archived original BitchX help documentation |
| `.cursor/rules/` | Cursor AI rules (Rust, C, clean-code, etc.) |
| `plugins/hello/` | Example "hello" plugin crate (`cdylib`) |
| `plugins/greet/` | Example "greet" plugin crate (`cdylib`) |
| `.cursor/skills/` | Cursor AI skills (karpathy, rust, typescript, etc.) |
