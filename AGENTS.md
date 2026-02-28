# AGENTS.md

## Project overview

BitchX is a terminal-based IRC (Internet Relay Chat) client. The original codebase is written in C (version 1.2c02, GNU Autotools build system). This project is being **rewritten in Rust**.

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

### Legacy C codebase (reference only)

The original C code serves as reference for the Rust rewrite.

**Building the C version** (for reference/comparison):
```
sudo apt-get install -y autoconf libncurses-dev
CFLAGS="-g -O2 -fcommon" ./configure --with-plugins --without-ssl
make
```

Caveats:
- `-fcommon` is required with GCC 10+ to avoid "multiple definition" linker errors.
- `--without-ssl` is needed on OpenSSL 3.x (the configure script checks for the removed `SSLeay` function).
- Binary: `source/BitchX`, plugins: `dll/*/*.so`.
- No automated test suite exists for the C codebase.

### Running the C version

```
./source/BitchX -n <nickname> <server>
```

To test locally: install `ngircd`, start it, then connect to `localhost`.

### Rust rewrite

When the Rust rewrite is underway:
- Build: `cargo build`
- Test: `cargo test`
- Lint: `cargo clippy`
- Format: `cargo fmt --check`
- All four must pass before code is considered complete.
- See `.cursor/rules/rust/main.mdc` for the full rule loading system (core, quality, features).

### Rust module structure

The Rust rewrite lives under `src/`. Modules being implemented in parallel (by different agents) include `irc/`, `ui/`, `plugin/`, `scripting/`, and `config`. Stubs exist for modules not yet implemented. When writing new code that depends on a stub, define compatible types locally or import from the stub and handle compilation gracefully.

### Crypto crate API notes

- `cbc` 0.1.2 with `cipher` 0.4.4: use `encrypt_padded_mut`/`decrypt_padded_mut` (not `*_vec_mut` variants). Import `BlockEncryptMut`/`BlockDecryptMut` from `cbc::cipher`.
- Blowfish minimum key length is 4 bytes.
- `aes-gcm` 0.10: use `Aes256Gcm::generate_nonce(&mut OsRng)` for nonce generation.

### Testing caveats

- All 248 tests pass reliably, including `irc::client` async tests with mock TCP servers.
- `cargo fmt --check` must pass. `cargo clippy` must produce zero warnings/errors.
- `aws-lc-sys` (transitive dep via `rustls`) requires `cmake` and a C compiler at build time. These are pre-installed in the VM.

### Key directories

| Directory | Purpose |
|-----------|---------|
| `source/` | Original C source files |
| `include/` | Original C header files |
| `dll/` | Original C plugin modules |
| `script/` | IRC scripts |
| `bitchx-docs/` | Help documentation |
| `.cursor/rules/` | Cursor AI rules (Rust, C, clean-code, etc.) |
| `.cursor/skills/` | Cursor AI skills (karpathy, rust, typescript, etc.) |
