#!/usr/bin/env bash
# Publishes to crates.io every crate `scripts/release-crates.sh pending` names: the listed crates whose workspace
# version crates.io does not have yet. One crate at a time, in scripts/publish-crates.txt order (each after the crates
# it depends on), with `cargo publish --no-verify --locked -p <crate>`; cargo waits for each to reach the index before
# returning, so the next one resolves against it.
#
#   cargo metadata --format-version 1 --no-deps --locked > metadata.json
#   scripts/publish-pending.sh <metadata.json>
#
# Prints `published <crate> <version>` per crate. crates.io rate-limits publishing (new crates especially): on an
# answer containing `429` or `Too Many Requests`, it waits $PUBLISH_BACKOFF seconds (default 600, crates.io's refill
# interval for new crates) and tries the same crate again, up to $PUBLISH_RETRIES times (default 36). Any other
# failure stops the run, so nothing publishes ahead of a crate it may depend on. A run that stopped can be run again:
# what is already on crates.io is no longer pending.
#
# CARGO_PUBLISH replaces `cargo publish` (the tests use a fake); it gets the same arguments. SPEEDYSPEC_REPO and
# SPEEDYSPEC_CRATES_INDEX are passed on to release-crates.sh.
set -euo pipefail

[ "$#" -eq 1 ] || { echo "usage: scripts/publish-pending.sh <metadata.json>" >&2; exit 2; }
metadata="$1"
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
backoff="${PUBLISH_BACKOFF:-600}"
retries="${PUBLISH_RETRIES:-36}"

publish() {
    if [ -n "${CARGO_PUBLISH:-}" ]; then
        "$CARGO_PUBLISH" "$@"
    else
        cargo publish "$@"
    fi
}

pending="$("$here/release-crates.sh" pending "$metadata")"
if [ -z "$pending" ]; then
    echo "every listed crate's version is already on crates.io"
    exit 0
fi
echo "publishing, in order:" $pending

log="$(mktemp)"
trap 'rm -f "$log"' EXIT
for name in $pending; do
    version="$(jq -r --arg n "$name" '.packages[] | select(.name == $n) | .version' "$metadata")"
    attempt=0
    while :; do
        status=0
        publish --no-verify --locked -p "$name" 2>&1 | tee "$log" || status=$?
        if [ "$status" -eq 0 ]; then
            echo "published $name $version"
            break
        fi
        if grep -q -E '(^|[^0-9.])429([^0-9.]|$)|Too Many Requests' "$log" && [ "$attempt" -lt "$retries" ]; then
            attempt=$((attempt + 1))
            echo "rate-limited publishing $name: waiting ${backoff}s (retry $attempt of $retries)"
            sleep "$backoff"
            continue
        fi
        echo "publishing $name $version failed (exit $status); stopping" >&2
        exit 1
    done
done
