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

# The fake records each attempt in $work/attempts, and behaves as $FAKE_MODE says for speedyspec-bbbb:
# `429` answers Too Many Requests on the first attempt only; `verify` always fails verification.
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
    output="$(PUBLISH_BACKOFF=0 PUBLISH_RETRIES=3 "$script" "$work/metadata.json" 2>&1)" || status=$?
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

# 4. Nothing pending: nothing is attempted, and it says so.
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
