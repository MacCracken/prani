#!/bin/sh
# check-alloc.sh -- every allocation prani makes goes through its funnel (2.2.0).
#
# ADR-0008: prani reports a refused allocation as PRANI_ERR_ALLOC, and it can only
# do that for allocations made through the three helpers in src/error.cyr
# (prani_alloc, prani_vec_new, prani_vec_push). A raw `alloc` returns 0, which the
# caller stores through; a raw `vec_new` returns 0, which the caller uses as a vec;
# a raw `vec_push` exits the process on a refused grow. tests/allocrefused.tcyr
# proves the helpers' callers report -- but only for the allocations it can see,
# which are exactly the ones made through the helpers. So this gate fails on any
# other allocating call in src/.
#
# Banned outside src/error.cyr: alloc vec_new vec_push and the allocator-taking
# forms the helpers wrap, alloc_via vec_new_a vec_push_a, called by any spelling
# the compiler accepts: `name(`, `name (`, the name at the end of one line with its
# `(` opening the next, and `&name` (a pointer to it, for fncall). Through 2.2.0 the
# gate matched only `name(` on one line, and the other three compiled and ran
# clean past it (audit D4, 2026-11-17). Comment text is ignored. In src/error.cyr
# three may each appear once, in their helper: prani_alloc_raw's alloc(size),
# prani_vec_new's vec_new_a and prani_vec_push's vec_push_a.
#
# Usage: sh scripts/check-alloc.sh   (from anywhere; exit 0 clean, 1 with a list)
set -eu
cd "$(dirname "$0")/.."

NAMES='(alloc|vec_new|vec_push|alloc_via|vec_new_a|vec_push_a)'
BAN="(^|[^A-Za-z0-9_])$NAMES[[:space:]]*\\("

# Code only: drop whole-line comments and trailing `# ...` comments. prani's
# sources hold no `#` inside a string literal on an allocating line; if one ever
# does, this over-strips and the gate under-reports, so it is checked below.
code() {
    sed -e 's/^[[:space:]]*#.*$//' -e 's/[[:space:]]#.*$//' "$1"
}

# hits FILE -- "line: text" for every banned call in FILE's code: on one line,
# split across lines (a blank or comment-only line between is skipped), or taken
# by address.
hits() {
    code "$1" | awk -v names="$NAMES" '
        { line[NR] = $0 }
        END {
            call = "(^|[^A-Za-z0-9_])" names "[[:space:]]*[(]"
            tail = "(^|[^A-Za-z0-9_])" names "[[:space:]]*$"
            addr = "&" names "([^A-Za-z0-9_]|$)"
            for (i = 1; i <= NR; i++) {
                s = line[i]
                if (s ~ call || s ~ addr) { print i ": " s; continue }
                if (s ~ tail) {
                    j = i + 1
                    while (j <= NR && line[j] ~ /^[[:space:]]*$/) j++
                    if (j <= NR && line[j] ~ /^[[:space:]]*[(]/) print i ": " s
                }
            }
        }'
}

fail=0
# Every .cyr under src/, at any depth: src/ is flat today, and a module moved into
# a subdirectory must not leave the gate by moving (audit 2026-11-17).
# The funnel is src/error.cyr and its generated f64 build, src/f64/error.cyr
# (ADR-0010): the same three calls, in each.
FUNNEL="src/error.cyr src/f64/error.cyr"
for f in $(find src -name '*.cyr' | sort); do
    case " $FUNNEL " in *" $f "*) continue ;; esac
    hits=$(hits "$f")
    if [ -n "$hits" ]; then
        echo "$hits" | sed "s|^|$f:|"
        fail=1
    fi
done

# Each funnel file: exactly one of each, the helpers' own calls.
for ff in $FUNNEL; do
    for pat in 'alloc\(size\)' \
               'vec_new_a\(prani_allocator\(\)\)' 'vec_push_a\(prani_allocator\(\), v, x\)'; do
        n=$(code "$ff" | grep -c -E "(^|[^A-Za-z0-9_])$pat" || true)
        if [ "$n" != "1" ]; then
            echo "$ff: expected exactly one '$pat', found $n"
            fail=1
        fi
    done
    others=$(hits "$ff" | grep -v -E 'alloc\(size\)|vec_new_a\(prani_allocator|vec_push_a\(prani_allocator' || true)
    if [ -n "$others" ]; then
        echo "$others" | sed "s|^|$ff:|"
        fail=1
    fi
done

# The comment stripping above must not be hiding a call behind a `#` in a string.
strs=$(find src -name '*.cyr' -exec grep -n -E '"[^"]*#[^"]*"' {} + | grep -E "$BAN" || true)
if [ -n "$strs" ]; then
    echo "a '#' inside a string on an allocating line; check-alloc.sh cannot read it:"
    echo "$strs"
    fail=1
fi

if [ "$fail" -ne 0 ]; then
    echo "check-alloc: allocate through prani_alloc / prani_vec_new / prani_vec_push (src/error.cyr, ADR-0008)"
    exit 1
fi
echo "check-alloc: every allocation in src/ goes through prani's funnel"
