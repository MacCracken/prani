#!/bin/sh
# run-examples.sh — build and run every example in docs/examples/.
#
# This is the gate roadmap 2.0.6 asks for: CI runs it on every push, so an
# example cannot rot against the API. An example that fails to build, or that
# exits non-zero, fails the build.
#
# Each example is a standalone program: it includes the dep bundles and
# dist/prani.cyr exactly as a consumer would, so building them also proves the
# published bundle is usable from outside the library.
#
# POSIX sh, deliberately — no arrays, no `shopt`, no `set -o pipefail`. CI runs
# this under dash, where `set -o pipefail` is an error, not a no-op. Keep it
# portable: if you reach for a bashism here, change the shebang too, or CI will
# fail on a line that works fine in your shell.
set -eu

EX_DIR="${EX_DIR:-docs/examples}"
OUT_DIR="${OUT_DIR:-build/examples}"

command -v cyrius >/dev/null 2>&1 || {
    echo "error: cyrius not on PATH. See CONTRIBUTING.md — Development Requirements." >&2
    exit 1
}

# Every `cyrius …` call re-resolves deps and races on cyrius.lock — serialize.
LOCK="${CYRIUS_LOCKFILE:-${TMPDIR:-/tmp}/prani-build.lock}"
cyr() {
    if command -v flock >/dev/null 2>&1; then
        flock "$LOCK" cyrius "$@"
    else
        cyrius "$@"
    fi
}

mkdir -p "$OUT_DIR"

total=0
fail=0
for src in "$EX_DIR"/*.cyr; do
    # An unmatched glob expands to the literal pattern in POSIX sh (there is no
    # nullglob), so skip anything that is not a real file rather than trying to
    # build "docs/examples/*.cyr".
    [ -f "$src" ] || continue
    total=$((total + 1))

    name=$(basename "$src" .cyr)
    bin="$OUT_DIR/$name"
    printf '── %s\n' "$name"

    if cyr build "$src" "$bin" >"$OUT_DIR/$name.build.log" 2>&1; then
        if "$bin"; then
            printf '   ok\n'
        else
            rc=$?
            echo "   RUN FAILED (exit $rc)" >&2
            fail=$((fail + 1))
        fi
    else
        echo "   BUILD FAILED — see $OUT_DIR/$name.build.log" >&2
        sed 's/^/   /' "$OUT_DIR/$name.build.log" >&2
        fail=$((fail + 1))
    fi
done

echo
if [ "$total" -eq 0 ]; then
    echo "error: no examples found in $EX_DIR/ — 2.0.6 requires five." >&2
    exit 1
fi
if [ "$fail" -ne 0 ]; then
    echo "examples: $fail of $total FAILED" >&2
    exit 1
fi
echo "examples: $total built and ran clean"
