BitchX 2.0.0-rs
===========================================================

BitchX is an IRC (Internet Relay Chat) client originally written by Colten
Edwards aka panasync@efnet. BitchX was originally based on ircII 2.8, and
later on the ircii-EPIC4 releases by Jeremy Nelson. It gained a following for
its speed, built-in scripting, DCC support, and the raw personality of its
interface. Development of the original C client wound down in the mid-2000s,
leaving a piece of IRC history dormant.

Now in 2026, BitchX is being brought back -- from scratch, in Rust.

I am Ayush, and I am taking an ambitious shot at rewriting BitchX completely
in Rust. The original client meant a lot to a generation of IRC users, and it
deserves to live again with modern TLS, a proper terminal UI, a safe and
testable codebase, and a plugin system that does not require patching C at
compile time. This is not a port -- it is a ground-up rewrite that preserves
the spirit of BitchX: fast, direct, scriptable, and unapologetic. I want to
bring this client back to the people who remember it and introduce it to
those who never had the chance.


Building BitchX-rs
==================

BitchX 2.0.0-rs is built with Cargo, the Rust package manager. You will need
Rust 1.75 or later, cmake, and a C compiler (required by the TLS dependency
at build time).

    $ cargo build --release

The compiled binary will be placed at target/release/bitchx. To build and
run the full workspace including plugins:

    $ cargo build --workspace

To run the test suite:

    $ cargo test --workspace

To check for lint warnings:

    $ cargo clippy

To verify formatting:

    $ cargo fmt --check

Installing BitchX-rs
====================

You can install BitchX-rs in multiple ways.

Method 1: Prebuilt release binaries (recommended)
-------------------------------------------------

Prebuilt binaries are published on every version tag (`v*`) in GitHub Releases.

1. Open the latest release:

       https://github.com/heyAyushh/BitchX-rs/releases/latest

2. Download the archive for your platform:
   - Linux: `bitchx-<tag>-linux-<arch>.tar.gz`
   - macOS: `bitchx-<tag>-macos-<arch>.tar.gz`
   - Windows: `bitchx-<tag>-windows-<arch>.zip`

3. Extract and place the binary in your `PATH`.

Linux/macOS example:

    $ tar -xzf bitchx-<tag>-linux-<arch>.tar.gz
    $ chmod +x bitchx
    $ sudo mv bitchx /usr/local/bin/bitchx

Windows PowerShell example:

    PS> Expand-Archive .\bitchx-<tag>-windows-<arch>.zip -DestinationPath .
    PS> Move-Item .\bitchx.exe "$env:USERPROFILE\AppData\Local\Microsoft\WindowsApps\bitchx.exe"

Method 2: Install directly with Cargo from Git
-----------------------------------------------

This method installs the latest code from the default branch:

    $ cargo install --git https://github.com/heyAyushh/bitchx-rs bitchx

To install from a specific tag:

    $ cargo install --git https://github.com/heyAyushh/bitchx-rs --tag <tag> bitchx

Method 3: Build from source
---------------------------

Clone and build manually:

    $ git clone https://github.com/heyAyushh/bitchx-rs.git
    $ cd bitchx-rs
    $ cargo build --release
    $ ./target/release/bitchx --help


Running BitchX-rs
=================

    $ bitchx --nick <yournick> --server <host[:port]>

By default BitchX connects over TLS on port 6697. To connect without TLS:

    $ bitchx --nick <yournick> --server <host> --no-tls

A configuration file can be placed at ~/.config/bitchx/bitchx.toml. Run
with --help for the full list of options.


Plugins
=======

BitchX-rs ships with a plugin system using a C ABI contract. Plugins are
shared libraries (.so files) loaded at runtime with /loaddll, unloaded with
/unloaddll, and listed with /listdll. Two example plugins, hello and greet,
are included in the plugins/ directory and built as part of the workspace.

To write a plugin, implement the bitchx_plugin_init, bitchx_plugin_on_message,
and bitchx_plugin_cleanup symbols as a cdylib crate. See plugins/hello/ for a
minimal example.


Workspace Layout
================

The codebase is organized as a Cargo workspace. The main bitchx crate lives
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
(bitchx-irc, bitchx-tui, bitchx-plugin-api, bitchx-scripting, bitchx-config)
so that parts of the BitchX stack can be reused independently.


Links
=====

    https://github.com/heyAyushh/BitchX-rs       Source repository
    https://www.bitchx.org/                        Original BitchX website
    https://faq.bitchx.org/                        Original FAQ


Contacts
========

    Maintainer: Ayush <heyayushh@gmail.com>

--
Last Updated:
Ayush
heyayushh@gmail.com
February 2026
