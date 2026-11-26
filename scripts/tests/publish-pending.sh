#!/usr/bin/env bash
# Tests scripts/publish-pending.sh with a fake `cargo publish` ($CARGO_PUBLISH) and a file:// sparse index
# ($SPEEDYSPEC_CRATES_INDEX), so nothing touches crates.io.
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
script="$here/../publish-pending.sh"
work="$(cd "$(mktemp -d)" && pwd -P)"
trap 'rm -rf "$work"' EXIT

failures=0
fail() {
    echo "FAIL: $*" >&2
    failures=$((failures + 1))
}

# A repository with three listed crates, and their metadata.
repo="$work/repo"
mkdir -p "$repo/scripts"
printf '%s\n' '# order' speedyspec-aaaa speedyspec-bbbb speedyspec-cccc >"$repo/scripts/publish-crates.txt"
cat >"$work/metadata.json" <<'JSON'
{"packages": [
  {"name": "speedyspec-aaaa", "version": "0.2.0", "publish": null, "dependencies": []},
  {"name": "speedyspec-bbbb", "version": "0.3.0", "publish": null, "dependencies": []},
  {"name": "speedyspec-cccc", "version": "0.4.0", "publish": null, "dependencies": []}
]}
JSON
export SPEEDYSPEC_REPO="$repo"

# The repository is a git checkout, with a release tag for each crate's version at HEAD.
git() { command git -C "$repo" -c user.name=test -c user.email=test@example.invalid -c commit.gpgsign=false -c tag.gpgsign=false "$@"; }
git init -q
git add scripts
git commit -q -m release
for tag in speedyspec-aaaa-v0.2.0 speedyspec-bbbb-v0.3.0 speedyspec-cccc-v0.4.0; do git tag "$tag"; done

# The fake records each attempt in $work/attempts, and behaves as $FAKE_MODE says for speedyspec-bbbb:
# `429` and `429-dated` answer Too Many Requests on the first attempt only; `verify` always fails verification;
# `exists` says the version is already on crates.io; `version-429` fails with a 429 that isn't an HTTP status;
# `network` and `502` fail with a connection reset or a Bad Gateway on the first attempt only.
cat >"$work/fake-publish" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
crate="${!#}"
echo "$crate $*" >>"$FAKE_WORK/attempts"
if [ "$crate" = speedyspec-bbbb ]; then
    case "$FAKE_MODE" in
        429)
            if [ ! -e "$FAKE_WORK/limited" ]; then
                touch "$FAKE_WORK/limited"
                echo "error: failed to publish to registry at https://crates.io" >&2
                echo "Caused by: the remote server responded with an error (status 429 Too Many Requests): You have published too many new crates in a short period of time" >&2
                exit 101
            fi
            ;;
        429-dated)
            if [ ! -e "$FAKE_WORK/limited" ]; then
                touch "$FAKE_WORK/limited"
                echo "Caused by: the remote server responded with an error (status 429 Too Many Requests): You have published too many new crates in a short period of time. Please try again after Wed, 26 Nov 2025 12:00:00 GMT and see https://crates.io/docs/rate-limits for more details." >&2
                exit 101
            fi
            ;;
        network)
            if [ ! -e "$FAKE_WORK/limited" ]; then
                touch "$FAKE_WORK/limited"
                echo "error: failed to publish speedyspec-bbbb v0.3.0 to registry at https://crates.io" >&2
                echo "Caused by:" >&2
                echo "  [35] SSL connect error (Recv failure: Connection reset by peer)" >&2
                exit 101
            fi
            ;;
        502)
            if [ ! -e "$FAKE_WORK/limited" ]; then
                touch "$FAKE_WORK/limited"
                echo "Caused by: the remote server responded with an error (status 502 Bad Gateway): upstream" >&2
                exit 101
            fi
            ;;
        exists)
            echo "error: crate speedyspec-bbbb@0.3.0 already exists on crates.io index" >&2
            exit 101
            ;;
        version-429)
            echo "error: failed to select a version for the requirement \`speedyspec-aaaa = \"^0.429.0\"\`" >&2
            exit 101
            ;;
        verify)
            echo "error: failed to verify package tarball" >&2
            exit 101
            ;;
    esac
fi
echo "   Uploaded $crate"
SH
chmod +x "$work/fake-publish"
export CARGO_PUBLISH="$work/fake-publish" FAKE_WORK="$work"

# run <mode> <index dir>: runs the script, leaving its output in $output and its exit status in $status.
run() {
    export FAKE_MODE="$1" SPEEDYSPEC_CRATES_INDEX="file://$2"
    rm -f "$work/attempts" "$work/limited"
    touch "$work/attempts"
    status=0
    output="$(PUBLISH_BACKOFF=0 PUBLISH_NET_BACKOFF=0 PUBLISH_RETRIES=3 "$script" "$work/metadata.json" 2>&1)" || status=$?
}

empty_index="$work/empty-index"
mkdir -p "$empty_index"

# 1. A 429 on the second crate: it waits $PUBLISH_BACKOFF (0) seconds, retries, and all three publish in order.
run 429 "$empty_index"
published="$(grep '^published ' <<<"$output" || true)"
want=$'published speedyspec-aaaa 0.2.0\npublished speedyspec-bbbb 0.3.0\npublished speedyspec-cccc 0.4.0'
if [ "$status" -ne 0 ]; then
    fail "429: exit $status (output: $output)"
elif [ "$published" != "$want" ]; then
    fail "429: published lines '$published', want '$want'"
elif ! grep -q 'rate-limited publishing speedyspec-bbbb: waiting 0s (retry 1 of 3)' <<<"$output"; then
    fail "429: no 0s backoff line (output: $output)"
elif [ "$(cut -d' ' -f1 "$work/attempts" | tr '\n' ' ')" != "speedyspec-aaaa speedyspec-bbbb speedyspec-bbbb speedyspec-cccc " ]; then
    fail "429: attempts $(cut -d' ' -f1 "$work/attempts" | tr '\n' ' ')"
elif ! grep -q -- '--no-verify --locked -p speedyspec-aaaa$' "$work/attempts"; then
    fail "429: arguments $(head -1 "$work/attempts")"
else
    echo "ok: 429 is retried after the backoff"
fi

# 1b. A 429 naming a retry time already past waits 1s, where GNU date can read the time; elsewhere, $PUBLISH_BACKOFF.
if date -u -d 'Wed, 26 Nov 2025 12:00:00 GMT' +%s >/dev/null 2>&1; then want_wait=1; else want_wait=0; fi
run 429-dated "$empty_index"
if [ "$status" -ne 0 ] || [ "$(grep -c '^published ' <<<"$output")" -ne 3 ]; then
    fail "429-dated: exit $status (output: $output)"
elif ! grep -q "rate-limited publishing speedyspec-bbbb: waiting ${want_wait}s (retry 1 of 3)" <<<"$output"; then
    fail "429-dated: want a ${want_wait}s wait (output: $output)"
else
    echo "ok: a 429's retry time is honoured (${want_wait}s)"
fi

# 1c. A network failure or a 5xx on the second crate is retried after $PUBLISH_NET_BACKOFF (0) seconds.
for mode in network 502; do
    run "$mode" "$empty_index"
    if [ "$status" -ne 0 ] || [ "$(grep -c '^published ' <<<"$output")" -ne 3 ]; then
        fail "$mode: exit $status (output: $output)"
    elif ! grep -q 'network error publishing speedyspec-bbbb: waiting 0s (retry 1 of 3)' <<<"$output"; then
        fail "$mode: no network retry line (output: $output)"
    elif [ "$(cut -d' ' -f1 "$work/attempts" | tr '\n' ' ')" != "speedyspec-aaaa speedyspec-bbbb speedyspec-bbbb speedyspec-cccc " ]; then
        fail "$mode: attempts $(cut -d' ' -f1 "$work/attempts" | tr '\n' ' ')"
    else
        echo "ok: $mode is retried"
    fi
done

# 2. Any other failure stops the run: exit 1, and the third crate is never attempted.
run verify "$empty_index"
if [ "$status" -ne 1 ]; then
    fail "verify: exit $status, want 1 (output: $output)"
elif grep -q speedyspec-cccc "$work/attempts"; then
    fail "verify: speedyspec-cccc was attempted"
elif grep -q '^published speedyspec-bbbb' <<<"$output"; then
    fail "verify: speedyspec-bbbb reported as published"
else
    echo "ok: another failure stops the run"
fi

# 3. A version already in the index is not attempted.
index="$work/index"
mkdir -p "$index/sp/ee"
echo '{"name":"speedyspec-aaaa","vers":"0.2.0","yanked":false}' >"$index/sp/ee/speedyspec-aaaa"
run none "$index"
if [ "$status" -ne 0 ]; then
    fail "published: exit $status (output: $output)"
elif grep -q speedyspec-aaaa "$work/attempts"; then
    fail "published: speedyspec-aaaa was attempted"
elif [ "$(grep -c '^published ' <<<"$output")" -ne 2 ]; then
    fail "published: want two published lines (output: $output)"
else
    echo "ok: a published version is skipped"
fi

# 5. crates.io already has the exact version: it counts as published, and the run goes on.
run exists "$empty_index"
if [ "$status" -ne 0 ]; then
    fail "exists: exit $status (output: $output)"
elif ! grep -q '^published speedyspec-bbbb 0.3.0 (already on crates.io)$' <<<"$output"; then
    fail "exists: no already-published line (output: $output)"
elif ! grep -q '^published speedyspec-cccc 0.4.0$' <<<"$output"; then
    fail "exists: speedyspec-cccc not published (output: $output)"
else
    echo "ok: a version crates.io already has counts as published"
fi

# 6. A "429" that isn't cargo's HTTP status line is an ordinary failure: no retry.
run version-429 "$empty_index"
if [ "$status" -ne 1 ] || grep -q 'rate-limited' <<<"$output" || [ "$(grep -c '^speedyspec-bbbb ' "$work/attempts")" -ne 1 ]; then
    fail "version-429: exit $status, attempts $(tr '\n' ' ' <"$work/attempts") (output: $output)"
else
    echo "ok: only cargo's status 429 line is a rate limit"
fi

# 7. A pending crate without its release tag is refused: nothing publishes, and the error names it.
git tag -d speedyspec-bbbb-v0.3.0 >/dev/null
run none "$empty_index"
if [ "$status" -ne 1 ] || [ -s "$work/attempts" ]; then
    fail "untagged: exit $status, attempts $(tr '\n' ' ' <"$work/attempts")"
elif ! grep -q 'speedyspec-bbbb 0.3.0 is not released: no tag speedyspec-bbbb-v0.3.0' <<<"$output"; then
    fail "untagged: no error naming speedyspec-bbbb (output: $output)"
else
    echo "ok: an untagged pending crate is refused"
fi

# 8. A tag that isn't HEAD or an ancestor of it is refused too.
git checkout -q -b side
git commit -q --allow-empty -m side
git tag speedyspec-bbbb-v0.3.0
git checkout -q -
run none "$empty_index"
if [ "$status" -ne 1 ] || [ -s "$work/attempts" ]; then
    fail "side tag: exit $status, attempts $(tr '\n' ' ' <"$work/attempts")"
elif ! grep -q 'tag speedyspec-bbbb-v0.3.0 is not HEAD or an ancestor of it' <<<"$output"; then
    fail "side tag: no error naming the tag (output: $output)"
else
    echo "ok: a tag off HEAD's history is refused"
fi
git tag -f speedyspec-bbbb-v0.3.0 HEAD >/dev/null

# 9. Nothing pending: nothing is attempted, and it says so.
for c in bbbb cccc; do
    echo "{\"name\":\"speedyspec-$c\",\"vers\":\"$(jq -r --arg n "speedyspec-$c" '.packages[] | select(.name == $n) | .version' "$work/metadata.json")\"}" >"$index/sp/ee/speedyspec-$c"
done
run none "$index"
if [ "$status" -ne 0 ] || [ -s "$work/attempts" ]; then
    fail "nothing pending: exit $status, attempts $(cat "$work/attempts")"
else
    echo "ok: nothing pending publishes nothing"
fi

if [ "$failures" -ne 0 ]; then
    echo "$failures failed" >&2
    exit 1
fi
echo "all passed"
