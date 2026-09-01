#!/usr/bin/env bash
# run-examples.sh — build and run every example in docs/examples/.
#
# This is the gate roadmap 2.0.6 asks for: CI runs it on every push, so an
# example cannot rot against the API. An example that fails to build, or that
# exits non-zero, fails the build.
#
# Each example is a standalone program: it includes the dep bundles and
# dist/prani.cyr exactly as a consumer would, so building them also proves the
# published bundle is usable from outside the library.
set -euo pipefail

EX_DIR="${EX_DIR:-docs/examples}"
OUT_DIR="${OUT_DIR:-build/examples}"

command -v cyrius >/dev/null 2>&1 || {
    echo "error: cyrius not on PATH. See CONTRIBUTING.md — Development Requirements." >&2
    exit 1
}

# Every `cyrius …` call re-resolves deps and races on cyrius.lock — serialize.
LOCK="${CYRIUS_LOCKFILE:-${TMPDIR:-/tmp}/prani-build.lock}"
cyr() {
    if command -v flock >/dev/null 2>&1; then flock "$LOCK" cyrius "$@"; else cyrius "$@"; fi
}

mkdir -p "$OUT_DIR"

shopt -s nullglob
examples=("$EX_DIR"/*.cyr)
shopt -u nullglob

if [ ${#examples[@]} -eq 0 ]; then
    echo "error: no examples found in $EX_DIR/ — 2.0.6 requires five." >&2
    exit 1
fi

fail=0
for src in "${examples[@]}"; do
    name="$(basename "$src" .cyr)"
    bin="$OUT_DIR/$name"
    printf '── %s\n' "$name"

    if ! cyr build "$src" "$bin" >"$OUT_DIR/$name.build.log" 2>&1; then
        echo "   BUILD FAILED — $OUT_DIR/$name.build.log" >&2
        sed 's/^/   /' "$OUT_DIR/$name.build.log" >&2
        fail=1
        continue
    fi

    if "$bin"; then
        printf '   ok\n'
    else
        rc=$?
        echo "   RUN FAILED (exit $rc)" >&2
        fail=1
    fi
done

echo
if [ "$fail" -ne 0 ]; then
    echo "examples: FAILED" >&2
    exit 1
fi
echo "examples: ${#examples[@]} built and ran clean"
