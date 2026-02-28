---
name: bash-expert
description: Expert Bash help for writing, debugging, and optimizing safe, portable shell one-liners and scripts on macOS/Linux. Use when users ask for Bash or POSIX sh commands, script reviews, ShellCheck fixes, quoting/pipeline issues, process/file automation, or converting requirements into shell scripts.
---

# Bash Expert

## Overview
- Write safe Bash scripts and one-liners with correct quoting, piping, and error handling.
- Produce portable code for macOS/Linux; call out GNU vs BSD differences and POSIX-only paths when needed.
- Debug issues (tracing, ShellCheck) and propose tests/dry-runs before destructive actions.

## Quick Start
1. Confirm shell/OS/permissions. Note macOS ships Bash 3.2 (no associative arrays); prefer Bash 4+ when available.
2. Choose shebang: `/usr/bin/env bash` for Bash 4+, `/bin/sh` for POSIX-only requirements.
3. Default safety (non-interactive):
   ```
   set -euo pipefail
   IFS=$'\n\t'
   shopt -s nullglob
   ```
   Avoid strict mode when partial failures are expected.
4. Quote everything (`"${var}"`), prefer `printf` over `echo`, and avoid word-splitting in loops.

## Workflow for Script Requests
1. Clarify goal, inputs, side effects, and scale (file counts, data size).
2. Identify tools (`find`, `grep`, `sed`, `awk`, `jq`, `tar`, `rsync`, etc.) and portability constraints.
3. Scaffold with logging and cleanup:
   ```
   #!/usr/bin/env bash
   set -euo pipefail
   IFS=$'\n\t'
   trap 'cleanup' EXIT
   cleanup(){ :; }
   log(){ printf '%s\n' "$*" >&2; }
   die(){ log "error: $*"; exit 1; }
   ```
4. Add argument parsing (`getopts`), sanity checks, and dry-run mode before mutating data.
5. Use safe loops (`while IFS= read -r line; do ...; done < file`) and null-delimited traversal (`find ... -print0 | while IFS= read -r -d '' path; do ...; done`).
6. Validate (`bash -n`, `shellcheck -x`), run with representative inputs, then present final command plus usage notes/rollback guidance.

## One-Liners & Pipelines
- Keep pipelines fail-fast with `set -o pipefail` or `bash -o pipefail -c 'cmd'`.
- Prefer process substitution over temp files when possible: `diff <(cmd1) <(cmd2)`.
- Use `xargs -0 -r` with `find -print0`; avoid `for f in $(...)`.
- Set `LC_ALL=C` for deterministic greps/sorts; specify PATH explicitly for cron/CI runs.

## Portability & Performance
- Bash-only features: arrays, `[[ ]]`, brace expansion, `**` globstar. Offer POSIX alternatives when `/bin/sh` is requested.
- BSD vs GNU flags: `sed -E` vs `-r`, `date` vs `gdate`, `stat` vs `gstat`; note fallbacks for macOS.
- Prefer built-ins where possible; for large data sets, use `awk`/`perl` instead of large Bash loops.

## Debugging & Validation
- Trace with `set -x` (or `bash -x script.sh`); use `PS4='+ ${BASH_SOURCE}:${LINENO}: '` for context.
- `shellcheck script.sh` for linting; address quoting and word-splitting warnings first.
- Log intermediates with `printf '%s\n' "msg: $var"`; disable tracing after debug.
- Provide dry-run flags and confirmation prompts before destructive actions.

## References
See `references/bash_patterns.md` for ready-to-use snippets: script skeletons, `getopts` template, safe `find`/`xargs`, `mktemp` + traps, heredocs, array handling, and bats test scaffolding. Load it only when deeper examples are needed.
