# Bash Patterns & Snippets

Use these when deeper examples are needed. Keep scripts short, quoted, and fail-fast.

## Script Skeleton
```
#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
trap 'cleanup' EXIT INT TERM
cleanup(){ :; }  # replace with actual clean-up
log(){ printf '%s\n' "$*" >&2; }
die(){ log "error: $*"; exit 1; }
```

## `getopts` Template
```
usage(){ printf 'Usage: %s [-n] -f file\n' "${0##*/}" >&2; exit 2; }
dry_run=0; file=
while getopts ':nf:' opt; do
  case "$opt" in
    n) dry_run=1 ;;
    f) file=$OPTARG ;;
    :) die "missing value for -$OPTARG" ;;
    \?) usage ;;
  esac
done
shift $((OPTIND-1))
[ -n "${file:-}" ] || usage
```

## Safe Iteration & Find/Xargs
```
find "$root" -type f -name '*.log' -print0 |
  while IFS= read -r -d '' path; do
    printf '%s\n' "$path"
  done
```
- Prefer `while IFS= read -r` over `for f in $(...)`.
- Use `xargs -0 -r` to avoid empty-input issues.

## Temporary Files/Dirs
```
tmpdir=$(mktemp -d) || exit 1
cleanup(){ rm -rf "$tmpdir"; }
trap cleanup EXIT INT TERM
```

## Heredoc Patterns
- Literal text: `cat <<'EOF' >file` (quoted delimiter stops expansion).
- Expand vars: `cat <<EOF >file` after validating inputs.

## Arrays & Expansions
```
items=(one "two three")
for item in "${items[@]}"; do
  printf '%s\n' "$item"
done
# Trim suffix: ${name%.*} ; replace: ${str/foo/bar}
```

## Locking & Concurrency
```
flock /tmp/my.lock -w 5 bash -c 'critical_section'
```
Use `set -C` + `>file` for simple lockfiles when `flock` unavailable.

## Debugging & Validation
- Syntax check: `bash -n script.sh`
- Lint: `shellcheck -x script.sh`
- Trace: `PS4='+ ${BASH_SOURCE}:${LINENO}: '; set -x`
- Time sections: `{ time cmd; } 2>&1 | log`

## macOS vs GNU Flags
- Prefer POSIX flags first; when GNU-only flags are needed, provide `brew install coreutils` fallback and use `gdate`, `gsed`, `gstat`.
- BSD `sed` uses `-E` instead of GNU `-r`.
- `xargs -r` is GNU-only; emulate with `xargs ${XARGS_OPTS:--0}` and guard with `if` when on macOS.

## Minimal bats Test
```
#!/usr/bin/env bats
setup(){ load 'test_helper/bats-support/load'; }
@test "script prints usage" {
  run ./script.sh
  [ "$status" -ne 0 ]
  [[ "$output" == *"Usage"* ]]
}
```

