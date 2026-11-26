#!/usr/bin/env bash
# The platform is closed: its crates build from this repository and crates.io alone, so each one can be published and
# a consumer gets exactly what was tested here.
#
#   cargo metadata --format-version 1 --locked > metadata.json      # with dependencies: no --no-deps
#   scripts/platform-closure.sh check <metadata.json> <repo-root>
#       Fails, with one line per problem:
#       - a dependency of any kind (normal, build or dev) that comes from anywhere but crates.io or a path under
#         <repo-root>/crates/:
#             <crate> depends on <dep> outside the platform (<path or source>)
#       - a path dependency that is not a workspace member (an excluded or stray crate):
#             <crate> depends on <dep> at <path>, which is not a workspace crate
#       - a package in the resolved graph built from a path outside <repo-root>/crates/. This is what a [patch] or
#         [replace] does, wherever it is declared (Cargo.toml or .cargo/config.toml, any table form):
#             <package> <version> is built from <dir>, outside the platform
#       - and, as a readable hint, the entries the root Cargo.toml and .cargo/config.toml declare:
#             Cargo.toml patches <name>        .cargo/config.toml replaces <name>
#   scripts/platform-closure.sh closure <metadata.json> <root>...
#       Prints the named crates closed under their normal and build path dependencies, one name per line, sorted:
#       what has to be published together with them. --no-deps metadata is enough.
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

# The [patch] and [replace] entries a TOML file declares, as "patches <name>" or "replaces <name>", one per line.
# Handles `[patch.<source>]` tables (names as keys), `[patch.<source>.<name>]` tables, a bare `[patch]` table with
# dotted keys (`crates-io.<name> = ...`), `[replace]` (`"<name>:<version>" = ...`) and `[replace."<name>:<version>"]`.
# Sources and keys may be quoted with " or '. The metadata check is the real guard; this only names what was declared.
declared_overrides() {
    awk '
        # Splits a dotted TOML key into out[1..n], dropping quotes; dots inside quotes do not split.
        function split_dotted(s, out,    n, i, c, q, cur) {
            n = 0; q = ""; cur = ""
            for (i = 1; i <= length(s); i++) {
                c = substr(s, i, 1)
                if (q != "") { if (c == q) q = ""; else cur = cur c; continue }
                if (c == "\"" || c == "\047") { q = c; continue }
                if (c == ".") { out[++n] = cur; cur = ""; continue }
                if (c ~ /[[:space:]]/) continue
                cur = cur c
            }
            out[++n] = cur
            return n
        }
        function crate(name) { sub(/:.*/, "", name); return name }
        /^[[:space:]]*\[/ {
            header = $0
            sub(/^[[:space:]]*\[+/, "", header)
            sub(/\]+[[:space:]]*(#.*)?$/, "", header)
            delete h
            n = split_dotted(header, h)
            mode = ""
            if (h[1] == "patch") {
                if (n == 1) mode = "bare-patch"
                else if (n == 2) mode = "patch"
                else print "patches " h[3]
            } else if (h[1] == "replace") {
                if (n == 1) mode = "replace"
                else print "replaces " crate(h[2])
            }
            next
        }
        mode != "" && /^[[:space:]]*[^#[:space:]][^=]*=/ {
            key = $0
            # The key ends at the first "=" outside quotes.
            q = ""; cut = 0
            for (i = 1; i <= length(key); i++) {
                c = substr(key, i, 1)
                if (q != "") { if (c == q) q = ""; continue }
                if (c == "\"" || c == "\047") { q = c; continue }
                if (c == "=") { cut = i; break }
            }
            if (cut == 0) next
            delete k
            nk = split_dotted(substr(key, 1, cut - 1), k)
            if (mode == "patch") print "patches " k[1]
            else if (mode == "replace") print "replaces " crate(k[1])
            else if (nk >= 2) print "patches " k[2]
            else print "patches everything in the " k[1] " source"
        }
    ' "$1"
}

case "$command" in
    check)
        [ "$#" -eq 1 ] || usage
        root="${1%/}"
        # cargo reports canonical paths (on macOS, /private/var for /var), so compare against the canonical root.
        if [ -d "$root" ]; then root="$(cd "$root" && pwd -P)"; fi
        jq -e '.resolve != null' "$metadata" >/dev/null || {
            echo "$metadata has no dependency graph: run cargo metadata without --no-deps" >&2
            exit 1
        }
        problems="$(jq -r --arg crates "$root/crates/" '
            (.workspace_members) as $members
            | ([.packages[] | select(.id as $id | $members | index($id)) | .manifest_path]) as $member_manifests
            | [ (.packages[] | select(.id as $id | $members | index($id)) as $p
                | $p.dependencies[] as $d
                | $d
                | if .source == null then
                      if .path == null or (.path | startswith($crates) | not) then
                          "\($p.name) depends on \(.name) outside the platform (\(.path // "no path or source"))"
                      elif ($member_manifests | index($d.path + "/Cargo.toml")) == null then
                          "\($p.name) depends on \(.name) at \(.path), which is not a workspace crate"
                      else empty end
                  elif .source == "registry+https://github.com/rust-lang/crates.io-index"
                      or .source == "sparse+https://index.crates.io/" then empty
                  else "\($p.name) depends on \(.name) outside the platform (\(.source))" end),
                (.packages[]
                | select(.source == null and (.manifest_path | startswith($crates) | not))
                | "\(.name) \(.version) is built from \(.manifest_path | rtrimstr("/Cargo.toml")), outside the platform")
            ] | unique[]
        ' "$metadata")"
        for file in Cargo.toml .cargo/config.toml .cargo/config; do
            [ -f "$root/$file" ] || continue
            while IFS= read -r entry; do
                problems+="${problems:+$'\n'}$file $entry"
            done < <(declared_overrides "$root/$file")
        done
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
