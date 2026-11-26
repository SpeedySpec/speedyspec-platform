#!/usr/bin/env bash
# Tests scripts/platform-closure.sh against the metadata fixtures in scripts/tests/fixtures/ (cut down from real
# `cargo metadata --no-deps` output, with the repository at /repo).
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
script="$here/../platform-closure.sh"
fixtures="$here/fixtures"
work="$(cd "$(mktemp -d)" && pwd -P)"
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

# Single-quoted sources, a bare [patch] with dotted keys, and [replace] in both forms.
printf '%s\n' "[patch.'https://example.invalid/x']" 'speedyspec-a = { path = "x" }' '' '[patch]' \
    'crates-io.speedyspec-b = { path = "x" }' "'https://example.invalid/y'.speedyspec-c = { path = \"x\" }" '' \
    '[replace]' '"speedyspec-d:0.1.0" = { path = "x" }' '' "[replace.'speedyspec-e:0.2.0']" 'path = "x"' \
    >"$root/Cargo.toml"
expect toml-forms 1 $'Cargo.toml patches speedyspec-a\nCargo.toml patches speedyspec-b\nCargo.toml patches speedyspec-c\nCargo.toml replaces speedyspec-d\nCargo.toml replaces speedyspec-e' \
    check "$work/closed.json" "$root"

# A patch in .cargo/config.toml is named too.
printf '%s\n' '[workspace]' >"$root/Cargo.toml"
mkdir -p "$root/.cargo"
printf '%s\n' '[patch.crates-io]' 'speedyspec-hooks = { path = "/elsewhere" }' >"$root/.cargo/config.toml"
expect config-patch 1 '.cargo/config.toml patches speedyspec-hooks' check "$work/closed.json" "$root"
rm -r "$root/.cargo"

# --no-deps metadata cannot show a patch's effect, so check refuses it.
jq '.resolve = null' "$fixtures/closed.json" >"$work/no-deps.json"
expect no-deps 1 "$work/no-deps.json has no dependency graph: run cargo metadata without --no-deps" \
    check "$work/no-deps.json" /repo

# A path dependency on a crate under crates/ that is not a workspace member (an excluded crate).
jq '(.packages[] | select(.name == "speedyspec-wp-html-syntax") | .dependencies) += [{"name": "speedyspec-wp-kses-testkit",
    "source": null, "req": "*", "kind": "dev", "path": "/repo/crates/speedyspec-wp-kses-testkit"}]' \
    "$fixtures/closed.json" >"$work/excluded.json"
expect excluded 1 \
    'speedyspec-wp-html-syntax depends on speedyspec-wp-kses-testkit at /repo/crates/speedyspec-wp-kses-testkit, which is not a workspace crate' \
    check "$work/excluded.json" /repo

# Real cargo: a member depends on itoa from crates.io, and each case swaps in a copy built from outside crates/ by a
# mechanism the text hint may not name. Only the graph's verdict is asserted.
if command -v cargo >/dev/null 2>&1; then
    itoa_version=1.0.19
    real_workspace() {
        local ws="$work/$1"
        mkdir -p "$ws/crates/member/src" "$ws/outside/itoa/src"
        printf '%s\n' '[workspace]' 'resolver = "3"' 'members = ["crates/*"]' >"$ws/Cargo.toml"
        printf '%s\n' '[package]' 'name = "member"' 'version = "0.1.0"' 'edition = "2024"' '' '[dependencies]' \
            "itoa = \"=$itoa_version\"" >"$ws/crates/member/Cargo.toml"
        : >"$ws/crates/member/src/lib.rs"
        printf '%s\n' '[package]' 'name = "itoa"' "version = \"$itoa_version\"" 'edition = "2021"' \
            >"$ws/outside/itoa/Cargo.toml"
        : >"$ws/outside/itoa/src/lib.rs"
        echo "$ws"
    }
    # expect_cargo <label> <workspace>: cargo honours the override, and check reports the outside build.
    expect_cargo() {
        local label="$1" ws="$2" output status=0
        if ! (cd "$ws" && cargo metadata --format-version 1 >"$ws/m.json" 2>"$ws/cargo.err"); then
            fail "$label: cargo metadata failed: $(cat "$ws/cargo.err")"
            return
        fi
        output="$("$script" check "$ws/m.json" "$ws" 2>&1)" || status=$?
        if [ "$status" != 1 ] || ! grep -qx "itoa $itoa_version is built from $ws/outside/itoa, outside the platform" <<<"$output"; then
            fail "$label: exit $status, output '$output'"
            return
        fi
        echo "ok: $label"
    }

    ws="$(real_workspace cargo-clean)"
    if (cd "$ws" && cargo metadata --format-version 1 >"$ws/m.json" 2>"$ws/cargo.err"); then
        expect cargo-clean 0 '' check "$ws/m.json" "$ws"

        ws="$(real_workspace cargo-dotted-patch)"
        printf '%s\n' '' '[patch]' 'crates-io.itoa = { path = "outside/itoa" }' >>"$ws/Cargo.toml"
        expect_cargo cargo-dotted-patch "$ws"

        ws="$(real_workspace cargo-config-patch)"
        mkdir -p "$ws/.cargo"
        printf '%s\n' '[patch.crates-io]' "itoa = { path = \"$ws/outside/itoa\" }" >"$ws/.cargo/config.toml"
        expect_cargo cargo-config-patch "$ws"

        ws="$(real_workspace cargo-replace)"
        printf '%s\n' '' '[replace]' "\"itoa:$itoa_version\" = { path = \"outside/itoa\" }" >>"$ws/Cargo.toml"
        expect_cargo cargo-replace "$ws"
    else
        fail "cargo-clean: cargo metadata failed (crates.io unreachable?): $(cat "$ws/cargo.err")"
    fi
else
    echo "skip: cargo cases (no cargo)"
fi

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
