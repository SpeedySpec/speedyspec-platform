#!/usr/bin/env bash
# The platform is closed: its crates build from this repository and crates.io alone, so each one can be published and
# a consumer gets exactly what was tested here.
#
#   cargo metadata --format-version 1 --no-deps > metadata.json
#   scripts/platform-closure.sh check <metadata.json> <repo-root>
#       Fails, with one line per problem, if a dependency of any kind (normal, build or dev) comes from anywhere but
#       crates.io or a path under <repo-root>/crates/:
#           <crate> depends on <dep> outside the platform (<path or source>)
#       or if <repo-root>/Cargo.toml has a [patch] entry:
#           Cargo.toml patches <name>
#   scripts/platform-closure.sh closure <metadata.json> <root>...
#       Prints the named crates closed under their normal and build path dependencies, one name per line, sorted:
#       what has to be published together with them.
set -euo pipefail

usage() {
    echo "usage: scripts/platform-closure.sh check <metadata.json> <repo-root>" >&2
    echo "       scripts/platform-closure.sh closure <metadata.json> <root>..." >&2
    exit 2
}

[ "$#" -ge 3 ] || usage
command="$1" metadata="$2"
shift 2

jq -e '.packages | type == "array"' "$metadata" >/dev/null 2>&1 || { echo "$metadata is not cargo metadata" >&2; exit 1; }

# The [patch] entries of a Cargo.toml, one name per line: `[patch.<source>]` tables list them as keys, and
# `[patch.<source>.<name>]` tables name one each. The source may be quoted (a URL).
patched_names() {
    awk '
        /^[[:space:]]*\[/ {
            header = $0
            sub(/^[[:space:]]*\[[[:space:]]*/, "", header)
            sub(/[[:space:]]*\][[:space:]]*(#.*)?$/, "", header)
            in_patch = 0
            if (header !~ /^patch\./) next
            rest = substr(header, 7)
            if (rest ~ /^"/) {
                rest = substr(rest, 2)
                rest = substr(rest, index(rest, "\"") + 1)
            } else if (index(rest, ".") > 0) {
                rest = substr(rest, index(rest, "."))
            } else {
                rest = ""
            }
            if (rest == "") { in_patch = 1; next }
            sub(/^\./, "", rest)
            gsub(/"/, "", rest)
            print rest
            next
        }
        in_patch && /^[[:space:]]*["A-Za-z0-9_-]+[[:space:]]*=/ {
            key = $0
            sub(/[[:space:]]*=.*/, "", key)
            gsub(/[[:space:]"]/, "", key)
            print key
        }
    ' "$1"
}

case "$command" in
    check)
        [ "$#" -eq 1 ] || usage
        root="${1%/}"
        problems="$(jq -r --arg crates "$root/crates/" '
            [ .packages[] as $p
              | $p.dependencies[]
              | if .source == null then
                    if .path != null and (.path | startswith($crates)) then empty
                    else "\($p.name) depends on \(.name) outside the platform (\(.path // "no path or source"))" end
                elif .source == "registry+https://github.com/rust-lang/crates.io-index"
                    or .source == "sparse+https://index.crates.io/" then empty
                else "\($p.name) depends on \(.name) outside the platform (\(.source))" end
            ] | unique[]
        ' "$metadata")"
        if [ -f "$root/Cargo.toml" ]; then
            while IFS= read -r name; do
                problems+="${problems:+$'\n'}Cargo.toml patches $name"
            done < <(patched_names "$root/Cargo.toml")
        fi
        if [ -n "$problems" ]; then
            printf '%s\n' "$problems" >&2
            exit 1
        fi
        ;;
    closure)
        roots="$(printf '%s\n' "$@" | jq -R . | jq -sc .)"
        jq -r --argjson roots "$roots" '
            (.packages | map({key: .name,
                value: [.dependencies[] | select(.path != null and (.kind == null or .kind == "build")) | .name]})
                | from_entries) as $graph
            | ($roots | map(select($graph[.] == null))) as $unknown
            | if ($unknown | length) > 0 then ($unknown[] | "\(.) is not a workspace crate\n" | halt_error(1))
              else reduce range(0; $graph | length) as $_
                  ($roots | unique; (. + [.[] | $graph[.][]?]) | unique)
                | .[]
              end
        ' "$metadata"
        ;;
    *)
        usage
        ;;
esac
