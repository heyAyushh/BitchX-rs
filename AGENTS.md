# AGENTS.md

## Cursor Cloud specific instructions

### Overview

BitchX is a terminal-based IRC (Internet Relay Chat) client written in C, using GNU Autotools for its build system. Version 1.2c02.

### Build prerequisites (system packages)

- `gcc`, `make`, `autoconf`, `libncurses-dev` (required)
- `libssl-dev` (optional; see SSL caveat below)

### Building

```
CFLAGS="-g -O2 -fcommon" ./configure --with-plugins --without-ssl
make
```

- **`-fcommon` is required** with GCC 10+. Without it, the linker will fail with "multiple definition" errors for global symbols like `no_hook_notify` and `serv_open_func`.
- The configure script checks for the legacy `SSLeay` function, which was removed in OpenSSL 3.x. Use `--without-ssl` on systems with OpenSSL 3+ to avoid configure failure.
- The binary is produced at `source/BitchX`.
- Plugins (`.so` files) are built in `dll/*/` subdirectories.

### Running

```
./source/BitchX -n <nickname> <server>
```

Use `-N` to skip auto-connect, `-d` for dumb terminal mode. See `./source/BitchX --help` for all flags.

To test locally, install `ngircd` and run it, then connect to `localhost`.

### Lint / Static analysis

No dedicated linter is configured. The build uses `-Wall` which produces many warnings (expected for this legacy C codebase). A clean build (zero errors) is the baseline.

### Tests

There is no automated test suite in this codebase. Verification is done by building and running the binary.

### Key directories

| Directory | Purpose |
|-----------|---------|
| `source/` | C source files and built binary |
| `include/` | Header files |
| `dll/` | Plugin modules |
| `script/` | IRC scripts |
| `bitchx-docs/` | Help documentation |
