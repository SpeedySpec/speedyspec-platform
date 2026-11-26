#!/usr/bin/env bash
# Publishes to crates.io every crate `scripts/release-crates.sh pending` names: the listed crates whose workspace
# version crates.io does not have yet, one at a time, in scripts/publish-crates.txt order (each after the crates it
# depends on), with `cargo publish --no-verify --locked -p <crate>`; cargo waits for each to reach the index before
# returning, so the next one resolves against it.
#
# Only released versions publish: each pending `<crate> <version>` needs the release tag `<crate>-v<version>` at HEAD
# or an ancestor of it. If any pending crate lacks one, nothing publishes, and it fails naming each such crate.
#
#   cargo metadata --format-version 1 --no-deps --locked > metadata.json
#   scripts/publish-pending.sh <metadata.json>
#
# Prints `published <crate> <version>` per crate. crates.io rate-limits publishing (new crates especially): on an
# answer with cargo's `status 429` error line, it waits until the time crates.io says to try again after (when GNU
# date can read it), or else $PUBLISH_BACKOFF seconds (default 600, crates.io's refill interval for new crates), and
# tries the same crate again, up to $PUBLISH_RETRIES times (default 36). crates.io saying the exact version already
# exists (another run got there first) counts as published. Any other failure stops the run, so nothing publishes
# ahead of a crate it may depend on. A run that stopped can be run again: what is already on crates.io is no longer
# pending.
#
# CARGO_PUBLISH replaces `cargo publish` (the tests use a fake); it gets the same arguments. SPEEDYSPEC_REPO (the git
# checkout whose tags are read; default this repository) and SPEEDYSPEC_CRATES_INDEX are passed on to
# release-crates.sh.
set -euo pipefail

[ "$#" -eq 1 ] || { echo "usage: scripts/publish-pending.sh <metadata.json>" >&2; exit 2; }
metadata="$1"
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo="${SPEEDYSPEC_REPO:-$(cd "$here/.." && pwd)}"
backoff="${PUBLISH_BACKOFF:-600}"
retries="${PUBLISH_RETRIES:-36}"

publish() {
    if [ -n "${CARGO_PUBLISH:-}" ]; then
        "$CARGO_PUBLISH" "$@"
    else
        cargo publish "$@"
    fi
}

# How long to wait after a 429: until the time crates.io names ("Please try again after <HTTP date>"), plus a few
# seconds, when GNU date can read it; else $PUBLISH_BACKOFF.
retry_wait() {
    local after target now
    after="$(sed -n 's/.*[Tt]ry again after \([A-Z][a-z][a-z], [0-9][0-9]* [A-Z][a-z][a-z] [0-9]* [0-9:]* GMT\).*/\1/p' "$1" | head -n 1)"
    if [ -n "$after" ] && target="$(date -u -d "$after" +%s 2>/dev/null)"; then
        now="$(date -u +%s)"
        if [ "$target" -le "$now" ]; then echo 1; else echo $((target - now + 5)); fi
    else
        echo "$backoff"
    fi
}

pending="$("$here/release-crates.sh" pending "$metadata")"
if [ -z "$pending" ]; then
    echo "every listed crate's version is already on crates.io"
    exit 0
fi

# Every pending version must be a release: its tag exists and HEAD contains it.
untagged=""
for name in $pending; do
    version="$(jq -r --arg n "$name" '.packages[] | select(.name == $n) | .version' "$metadata")"
    tag="$name-v$version"
    commit="$(git -C "$repo" rev-parse -q --verify "refs/tags/$tag^{commit}" 2>/dev/null)" || commit=""
    if [ -z "$commit" ]; then
        echo "::error::$name $version is not released: no tag $tag" >&2
        untagged="$untagged $name"
    elif ! git -C "$repo" merge-base --is-ancestor "$commit" HEAD; then
        echo "::error::$name $version is not released here: tag $tag is not HEAD or an ancestor of it" >&2
        untagged="$untagged $name"
    fi
done
if [ -n "$untagged" ]; then
    echo "nothing is published: unreleased versions of$untagged" >&2
    exit 1
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
        # cargo's error line for an HTTP 429: "the remote server responded with an error (status 429 Too Many
        # Requests)".
        if grep -q -E 'status 429([^0-9]|$)' "$log" && [ "$attempt" -lt "$retries" ]; then
            attempt=$((attempt + 1))
            wait="$(retry_wait "$log")"
            echo "rate-limited publishing $name: waiting ${wait}s (retry $attempt of $retries)"
            sleep "$wait"
            continue
        fi
        # This exact version is already on crates.io (cargo: "crate <name>@<version> already exists on crates.io
        # index"; the registry: "crate version `<version>` is already uploaded").
        if grep -q -F -e "$name@$version already exists" -e "crate version \`$version\` is already uploaded" "$log"; then
            echo "published $name $version (already on crates.io)"
            break
        fi
        echo "publishing $name $version failed (exit $status); stopping" >&2
        exit 1
    done
done
