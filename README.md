BitchY
======

BitchY is an unofficial Rust IRC client inspired by the original BitchX.
It is a separate project with its own name, binary, config paths, and plugin
ABI. It is not affiliated with or endorsed by Colten Edwards, the original
BitchX project, or the BitchX 2 effort.

This repository also includes archival documentation from the historical
BitchX project under `bitchx-docs/`. Those files intentionally keep their
original names and wording.


Building BitchY
===============

BitchY is built with Cargo, the Rust package manager. You will need Rust
1.75 or later, cmake, and a C compiler (required by the TLS dependency at
build time).

    $ cargo build --release

The compiled binary will be placed at `target/release/bitchy`. To build and
run the full workspace including plugins:

    $ cargo build --workspace

To run the test suite:

    $ cargo test --workspace

To check for lint warnings:

    $ cargo clippy --workspace

To verify formatting:

    $ cargo fmt --check


Installing BitchY
=================

You can install BitchY in three ways. The recommended method is using
prebuilt binaries from GitHub Releases, published on every version tag (`v*`):

    https://github.com/heyAyushh/BitchX-rs/releases/latest

Download the archive that matches your platform (`bitchy-<tag>-linux-<arch>.tar.gz`,
`bitchy-<tag>-macos-<arch>.tar.gz`, or `bitchy-<tag>-windows-<arch>.zip`), extract
it, and place the binary in your `PATH`.

Linux/macOS example:

    $ tar -xzf bitchy-<tag>-linux-<arch>.tar.gz
    $ chmod +x bitchy
    $ sudo mv bitchy /usr/local/bin/bitchy

Windows PowerShell example:

    PS> Expand-Archive .\bitchy-<tag>-windows-<arch>.zip -DestinationPath .
    PS> Move-Item .\bitchy.exe "$env:USERPROFILE\AppData\Local\Microsoft\WindowsApps\bitchy.exe"

If you prefer installing through Cargo directly from Git, use:

    $ cargo install --git https://github.com/heyAyushh/bitchx-rs bitchy

To install a specific tagged release with Cargo, use:

    $ cargo install --git https://github.com/heyAyushh/bitchx-rs --tag <tag> bitchy

If you want full control, build from source:

    $ git clone https://github.com/heyAyushh/bitchx-rs.git
    $ cd bitchx-rs
    $ cargo build --release
    $ ./target/release/bitchy --help


Running BitchY
==============

    $ bitchy --nick <yournick> --server <host[:port]>

By default BitchY connects over TLS on port 6697. To connect without TLS:

    $ bitchy --nick <yournick> --server <host> --no-tls

A configuration file can be placed at `~/.config/bitchy/bitchy.toml`. Run
with `--help` for the full list of options.


Plugins
=======

BitchY ships with a plugin system using a C ABI contract. Plugins are shared
libraries (`.so` files) loaded at runtime with `/loaddll`, unloaded with
`/unloaddll`, and listed with `/listdll`. Two example plugins, `hello` and
`greet`, are included in the `plugins/` directory and built as part of the
workspace.

To write a plugin, implement the `bitchy_plugin_name`,
`bitchy_plugin_version`, `bitchy_plugin_description`, `bitchy_plugin_init`,
`bitchy_plugin_cleanup`, and optional `bitchy_plugin_on_message` symbols as a
`cdylib` crate. See `plugins/hello/` for a minimal example.


Workspace Layout
================

The codebase is organized as a Cargo workspace. The main `bitchy` crate lives
at the root and is structured as follows:

    src/irc/        IRC connection, protocol, commands, channels, users,
                    CTCP, DCC, flood control, modes, and encryption.
    src/ui/         Terminal UI: application loop, input handling, theme,
                    widgets, and ANSI startup art.
    src/plugin/     C ABI plugin loader and helper macros.
    src/scripting/  Alias and scripting engine.
    src/config.rs   Configuration loading and defaults.
    src/error.rs    Shared error types.
    plugins/hello/  Example hello plugin crate.
    plugins/greet/  Example greet plugin crate.

The intent going forward is to break these into separate published crates
(`bitchy-irc`, `bitchy-tui`, `bitchy-plugin-api`, `bitchy-scripting`,
`bitchy-config`) so that parts of the BitchY stack can be reused
independently.


Licensing
=========

This repository is distributed under the BSD-3-Clause license. The full text
is in `LICENSE`, and `COPYRIGHT` records the original BitchX attribution and
the Rust rewrite attribution used by this repository.


Links
=====

    https://github.com/heyAyushh/BitchX-rs       Current repository location
    https://www.bitchx.org/                      Original BitchX website
    https://faq.bitchx.org/                      Original FAQ


Contacts
========

    Maintainer: Ayush
