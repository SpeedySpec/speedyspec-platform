#!/usr/bin/env bash
# Tests scripts/platform-closure.sh against the metadata fixtures in scripts/tests/fixtures/ (cut down from real
# `cargo metadata --no-deps` output, with the repository at /repo).
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
script="$here/../platform-closure.sh"
fixtures="$here/fixtures"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

failures=0
fail() {
    echo "FAIL: $*" >&2
    failures=$((failures + 1))
}

# expect <label> <exit status> <exact output or -> <args...>
expect() {
    local label="$1" want_status="$2" want_output="$3" output status=0
    shift 3
    output="$("$script" "$@" 2>&1)" || status=$?
    if [ "$status" != "$want_status" ]; then
        fail "$label: exit $status, want $want_status (output: $output)"
        return
    fi
    if [ "$want_output" != "-" ] && [ "$output" != "$want_output" ]; then
        fail "$label: output '$output', want '$want_output'"
        return
    fi
    echo "ok: $label"
}

expect closed 0 '' check "$fixtures/closed.json" /repo

expect closed-trailing-slash 0 '' check "$fixtures/closed.json" /repo/

expect open 1 \
    'speedyspec-wp-html-syntax depends on speedyspec-wp-kses-testkit outside the platform (/speedyspec-server/crates/speedyspec-wp-kses-testkit)' \
    check "$fixtures/open.json" /repo

expect git-dep 1 \
    'speedyspec-wp-kses-contracts depends on speedyspec-hooks outside the platform (git+https://github.com/SpeedySpec/speedyspec-server?branch=main#0123456789abcdef0123456789abcdef01234567)' \
    check "$fixtures/git-dep.json" /repo

# A repository root of /rep must not accept paths under /repo.
expect prefix-boundary 1 - check "$fixtures/closed.json" /rep

# The same closed metadata, under a root whose Cargo.toml patches a crate.
root="$work/repo"
mkdir -p "$root"
cp "$fixtures/patch.toml" "$root/Cargo.toml"
sed "s|/repo/|$root/|g" "$fixtures/closed.json" >"$work/closed.json"
expect patch 1 'Cargo.toml patches speedyspec-hooks' check "$work/closed.json" "$root"

# The table form, and a patch of a git source.
printf '%s\n' '[workspace]' '' '[patch."https://github.com/SpeedySpec/speedyspec-server"]' \
    'speedyspec-hooks = { path = "x" }' '' '[patch.crates-io.speedyspec-cqrs]' 'path = "y"' '' '[profile.release]' \
    'lto = true' >"$root/Cargo.toml"
expect patch-forms 1 $'Cargo.toml patches speedyspec-hooks\nCargo.toml patches speedyspec-cqrs' \
    check "$work/closed.json" "$root"

expect closure 0 $'speedyspec-wp-html-syntax\nspeedyspec-wp-kses-contracts\nspeedyspec-wp-kses-lexical\nspeedyspec-wp-token-map' \
    closure "$fixtures/closed.json" speedyspec-wp-kses-lexical

# Dev-dependencies do not join the closure; several roots merge.
expect closure-dev 0 $'speedyspec-wp-html-split\nspeedyspec-wp-kses-contracts' \
    closure "$fixtures/closed.json" speedyspec-wp-html-split speedyspec-wp-kses-contracts

expect closure-unknown 1 'speedyspec-nope is not a workspace crate' closure "$fixtures/closed.json" speedyspec-nope

expect usage 2 - frobnicate "$fixtures/closed.json"

if [ "$failures" -ne 0 ]; then
    echo "$failures failed" >&2
    exit 1
fi
echo "all passed"
