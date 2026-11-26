#!/usr/bin/env bash
# Every crate in scripts/publish-crates.txt packages what crates.io and docs.rs need, and nothing they must not get.
#
#   scripts/check-crate-packages.sh
#
# For each listed crate, `cargo package --list` must include LICENSE and README.md and must not ship tests/ (the tests
# read this repository's compat/ and tests/php). Its manifest, read through `cargo metadata`, must carry a
# description, license MIT, the platform's repository, readme README.md, one to five keywords, at least one category,
# rust-version 1.99, edition 2024 and a [package.metadata.docs.rs] table.
#
# Prints one line per problem, `<crate>: <problem>`, and exits 1 if there is any. SPEEDYSPEC_REPO overrides the
# repository root.
set -euo pipefail

repo="${SPEEDYSPEC_REPO:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
list_file="$repo/scripts/publish-crates.txt"
[ -f "$list_file" ] || { echo "no $list_file" >&2; exit 1; }
repository="https://github.com/SpeedySpec/speedyspec-platform"

metadata="$(mktemp)"
files="$(mktemp)"
trap 'rm -f "$metadata" "$files"' EXIT
(cd "$repo" && cargo metadata --format-version 1 --no-deps) > "$metadata"

failed=0
for name in $(grep -v -e '^[[:space:]]*$' -e '^[[:space:]]*#' "$list_file"); do
    problems="$(jq -r --arg n "$name" --arg repository "$repository" '
        [.packages[] | select(.name == $n)] as $found
        | if ($found | length) == 0 then "\($n): not a workspace crate"
          else $found[0] as $p
            | (if ($p.description // "") == "" then "\($n): no description" else empty end),
              (if $p.license != "MIT" then "\($n): license is \($p.license // "unset"), not MIT" else empty end),
              (if $p.repository != $repository then "\($n): repository is \($p.repository // "unset")" else empty end),
              (if (($p.readme // "") | test("(^|/)README\\.md$") | not) then "\($n): readme is \($p.readme // "unset"), not README.md" else empty end),
              (if (($p.keywords | length) < 1 or ($p.keywords | length) > 5) then "\($n): \($p.keywords | length) keywords, not 1-5" else empty end),
              (if ($p.categories | length) < 1 then "\($n): no categories" else empty end),
              (if $p.rust_version != "1.99" then "\($n): rust-version is \($p.rust_version // "unset"), not 1.99" else empty end),
              (if $p.edition != "2024" then "\($n): edition is \($p.edition), not 2024" else empty end),
              (if ($p.metadata.docs.rs? // null) == null then "\($n): no [package.metadata.docs.rs]" else empty end)
          end
    ' "$metadata")"

    if (cd "$repo" && cargo package --list --allow-dirty -p "$name") > "$files" 2>/dev/null; then
        grep -qx LICENSE "$files" || problems+=$'\n'"$name: no LICENSE"
        grep -qx README.md "$files" || problems+=$'\n'"$name: no README.md"
        if grep -q '^tests/' "$files"; then problems+=$'\n'"$name: ships tests/"; fi
    else
        problems+=$'\n'"$name: cargo package --list failed"
    fi

    problems="$(sed '/^$/d' <<<"$problems")"
    if [ -n "$problems" ]; then
        printf '%s\n' "$problems"
        failed=1
    fi
done
exit "$failed"
