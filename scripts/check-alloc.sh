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
# Banned outside src/error.cyr: alloc( vec_new( vec_push( and the allocator-taking
# forms the helpers wrap: alloc_via( vec_new_a( vec_push_a(. Comment text is
# ignored. In src/error.cyr three may each appear once, in their helper:
# prani_alloc_raw's alloc(size), prani_vec_new's vec_new_a and prani_vec_push's
# vec_push_a.
#
# Usage: sh scripts/check-alloc.sh   (from anywhere; exit 0 clean, 1 with a list)
set -eu
cd "$(dirname "$0")/.."

BAN='(^|[^A-Za-z0-9_])(alloc|vec_new|vec_push|alloc_via|vec_new_a|vec_push_a)\('

# Code only: drop whole-line comments and trailing `# ...` comments. prani's
# sources hold no `#` inside a string literal on an allocating line; if one ever
# does, this over-strips and the gate under-reports, so it is checked below.
code() {
    sed -e 's/^[[:space:]]*#.*$//' -e 's/[[:space:]]#.*$//' "$1"
}

fail=0
for f in src/*.cyr; do
    [ "$f" = "src/error.cyr" ] && continue
    hits=$(code "$f" | grep -n -E "$BAN" || true)
    if [ -n "$hits" ]; then
        echo "$hits" | sed "s|^|$f:|"
        fail=1
    fi
done

# src/error.cyr: exactly one of each, the helpers' own calls.
for pat in 'alloc\(size\)' \
           'vec_new_a\(prani_allocator\(\)\)' 'vec_push_a\(prani_allocator\(\), v, x\)'; do
    n=$(code src/error.cyr | grep -c -E "(^|[^A-Za-z0-9_])$pat" || true)
    if [ "$n" != "1" ]; then
        echo "src/error.cyr: expected exactly one '$pat', found $n"
        fail=1
    fi
done
others=$(code src/error.cyr | grep -n -E "$BAN" | grep -v -E 'alloc\(size\)|vec_new_a\(prani_allocator|vec_push_a\(prani_allocator' || true)
if [ -n "$others" ]; then
    echo "$others" | sed 's|^|src/error.cyr:|'
    fail=1
fi

# The comment stripping above must not be hiding a call behind a `#` in a string.
strs=$(grep -n -E '"[^"]*#[^"]*"' src/*.cyr | grep -E "$BAN" || true)
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
