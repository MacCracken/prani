#!/bin/sh
# gen-f64.sh -- write the f64 build of every real-source module (ADR-0010).
#
#   sh scripts/gen-f64.sh           regenerate src/f64/<module>.cyr
#   sh scripts/gen-f64.sh --check   fail if src/f64/ is not what this would write
#
# prani's source is written once, over src/real.cyr's primitives (pr_mul, pr_lt, ...).
# The f32 build compiles that source as it is: each primitive is an #inline f32 op.
# The f64 build could do the same, but cycc 6.7.18 copies every argument of an
# #inline replay into a slot of its own, and through the primitives the f64 build
# measured 1.09x 2.2.1 (emotion_evaluate 1.30x). So the f64 build compiles this
# script's output instead: the same source with each primitive spelled as the f64
# builtin it stands for, which is the code 2.2.1 compiled.
#
# A module is a real source when it opens with the guard pair
#     # prani:real-source
#     #ifdef PRANI_F32
# and closes with
#     # prani:real-source-end
#     #endif
# Its f64 build is src/f64/<module>.cyr: the same text between the guards, wrapped
# in `#ifndef PRANI_F32`, with these rewrites and nothing else:
#   1. each `#ifdef PRANI_F32 ... #else ... #endif` block keeps only its #else arm;
#   2. in code (never in a comment, a string or a directive line), pr_<op>( becomes
#      f64_<op>( for the primitives with an f64 builtin of the same meaning;
#   3. pr_to64( and pr_of64( become a bare `(`: in the f64 build they are the
#      identity;
#   4. a run of blank lines left where an f32 arm was dropped becomes one.
# A module with no guard has no reals to rewrite and is shared by both builds.
#
# Both modes also check, and fail on (2.3.2, audit A1/A2/A4):
#   - a directive the generator cannot read: one is recognised by its first word, so
#     `#else   # a comment` is an #else; an #elif in a PRANI_F32 block, or a block
#     left open at the end of a module, is refused rather than guessed at;
#   - [lib].modules not listing every real source immediately followed by its twin,
#     src/f64/<module>.cyr: without the twin the f64 bundle lacks the module, and
#     without the source the f32 bundle does;
#   - the FORMAT LINT: in code the f32 build compiles outside a PRANI_F32 block (a
#     real source's shared body, or a module with no guard), an f64_* call, an F64_*
#     constant or a 16-digit hex literal not inside pr_of64(...) is an f64 value
#     handed to f32 code. A deliberate one carries `# f64-ok: <why>` on its line.
set -eu
cd "$(dirname "$0")/.."

check=0
[ "${1:-}" = "--check" ] && check=1

out=src/f64
if [ $check -eq 1 ]; then
    out=$(mktemp -d)
    trap 'rm -rf "$out"' EXIT
fi
mkdir -p "$out"

# Shared by the generator and the format lint. walk(s, dorew) scans one line with
# the string state INSTR carried across lines (a string may span lines): WOUT is the
# line's code with pr_ rewritten outside strings when dorew is set, WCOM its comment
# (from the first # outside a string), WNOSTR its code with every string removed.
AWKLIB='
function trim(s) { sub(/^[ \t]+/, "", s); sub(/[ \t]+$/, "", s); return s }
function dir_words(t,    w, n) {
    n = split(t, w, /[ \t]+/)
    DW1 = w[1]; DW2 = (n >= 2) ? w[2] : ""; DW3 = (n >= 3) ? w[3] : ""
}
function rest_ok(x) { return (x == "" || substr(x, 1, 1) == "#") }
function rewrite(s,    r, i, j, pre, name, c) {
    r = ""
    while ((i = index(s, "pr_")) > 0) {
        pre = (i == 1) ? "" : substr(s, i - 1, 1)
        j = i + 3; name = ""
        while (j <= length(s)) {
            c = substr(s, j, 1)
            if (c !~ /[a-z0-9_]/) break
            name = name c; j++
        }
        if (pre !~ /[A-Za-z0-9_]/ && substr(s, j, 1) == "(" && (name in op)) {
            r = r substr(s, 1, i - 1) "f64_" name "("
        } else if (pre !~ /[A-Za-z0-9_]/ && substr(s, j, 1) == "(" && (name == "to64" || name == "of64")) {
            r = r substr(s, 1, i - 1) "("
        } else {
            r = r substr(s, 1, j - 1); s = substr(s, j); continue
        }
        s = substr(s, j + 1)
    }
    return r s
}
function walk(s, dorew,    i, c, o, seg, nostr) {
    o = ""; seg = ""; nostr = ""; WCOM = ""
    i = 1
    while (i <= length(s)) {
        c = substr(s, i, 1)
        if (INSTR) {
            if (c == "\\") { o = o c substr(s, i + 1, 1); i += 2; continue }
            o = o c
            if (c == "\"") INSTR = 0
            i++; continue
        }
        if (c == "\"") {
            o = o (dorew ? rewrite(seg) : seg) c; nostr = nostr seg " "; seg = ""; INSTR = 1; i++; continue
        }
        if (c == "#") { WCOM = substr(s, i); break }
        seg = seg c; i++
    }
    WOUT = o (dorew ? rewrite(seg) : seg); WNOSTR = nostr seg
}
BEGIN {
    split("add sub mul div neg abs lt le gt ge eq clamp min max sin cos sqrt pow cbrt floor trunc from to", ops, " ")
    for (k in ops) op[ops[k]] = 1
    INSTR = 0
}
'

gen() {
    awk -v SRC="$1" "$AWKLIB"'
    BEGIN { state = 0; depth = 0; bad = 0; nb = 0; nh = 0; gap = 0 }
    function emit(x,    blank) {
        blank = (x ~ /^[ \t]*$/)
        if (blank && nb > 0 && body[nb] ~ /^[ \t]*$/ && gap) return
        body[++nb] = x
        if (!blank) gap = 0
    }
    function fail(code, why) { bad = code; printf "gen-f64: %s:%d: %s\n", SRC, FNR, why > "/dev/stderr"; exit 1 }
    {
        line = $0
        skipping = 0
        for (k = 1; k <= depth; k++) if (stk[k] == "f32") skipping = 1
        if (INSTR) {                            # a line inside a multi-line string
            walk(line, 1)
            if (state == 2 && !skipping) emit(WOUT WCOM); else if (skipping) gap = 1
            if (state < 2) hdr[++nh] = line
            next
        }
        t = trim(line)
        if (state == 0) {                       # before the guard: the header comment
            if (t == "# prani:real-source") { state = 1; next }
            hdr[++nh] = line; next
        }
        if (state == 1) {                       # the guard itself
            dir_words(t)
            if (DW1 != "#ifdef" || DW2 != "PRANI_F32" || !rest_ok(DW3)) fail(1, "the guard must be #ifdef PRANI_F32")
            state = 2; next
        }
        if (state == 3) {                       # the closing guard
            if (t == "") next
            dir_words(t)
            if (DW1 != "#endif" || !rest_ok(DW2)) fail(2, "the closing guard must be #endif")
            state = 4; next
        }
        if (state == 4) { if (t != "") fail(3, "text after the closing guard"); next }
        if (t == "# prani:real-source-end") {
            if (depth != 0) fail(4, "a conditional block is still open at the end guard")
            state = 3; next
        }
        # state 2: the body
        if (substr(t, 1, 1) == "#") {
            dir_words(t)
            if (DW1 == "#if" || DW1 == "#ifdef" || DW1 == "#ifndef") {
                isf32 = (DW1 == "#ifdef" && DW2 == "PRANI_F32")
                if (isf32 && !rest_ok(DW3)) fail(5, "#ifdef PRANI_F32 followed by more than a comment")
                stk[++depth] = isf32 ? "f32" : "other"
                if (isf32 || skipping) { gap = 1; next }
                emit(line); next
            }
            if (DW1 == "#elif") {
                if (depth > 0 && stk[depth] != "other") fail(6, "#elif in a PRANI_F32 block")
                if (skipping) { gap = 1; next }
                emit(line); next
            }
            if (DW1 == "#else") {
                if (!rest_ok(DW2)) fail(7, "#else followed by more than a comment")
                if (depth == 0) fail(8, "#else with no open block")
                if (stk[depth] == "f32") { stk[depth] = "f64"; gap = 1; next }
                if (stk[depth] == "f64") fail(9, "a second #else in a PRANI_F32 block")
                if (skipping) { gap = 1; next }
                emit(line); next
            }
            if (DW1 == "#endif") {
                if (!rest_ok(DW2)) fail(10, "#endif followed by more than a comment")
                if (depth == 0) fail(11, "#endif with no open block")
                d = stk[depth]; depth--
                if (d == "f32" || d == "f64" || skipping) { gap = 1; next }
                emit(line); next
            }
            if (skipping) { gap = 1; next }
            emit(line); next                    # a comment or another directive
        }
        if (skipping) { walk(line, 0); gap = 1; next }
        walk(line, 1)
        emit(WOUT WCOM)
    }
    END {
        if (bad) exit 1
        if (state != 4) { printf "gen-f64: %s: malformed guard (state %d)\n", SRC, state > "/dev/stderr"; exit 1 }
        if (INSTR) { printf "gen-f64: %s: a string is still open at the end\n", SRC > "/dev/stderr"; exit 1 }
        for (k = 1; k <= nh; k++) print hdr[k]
        print "#"
        print "# GENERATED by scripts/gen-f64.sh from " SRC " -- do not edit; edit the source and"
        print "# re-run the script. This is the f64 build of that module (ADR-0010)."
        print "#ifndef PRANI_F32"
        for (k = 1; k <= nb; k++) print body[k]
        print "#endif"
    }' "$1"
}

# The format lint (see the header): f64 values in code the f32 build compiles.
lint() {
    awk -v SRC="$1" -v GUARDED="$2" "$AWKLIB"'
    BEGIN { depth = 0; inguard = (GUARDED == "1") ? 0 : 1; bad = 0 }
    {
        line = $0
        inarm = 0
        for (k = 1; k <= depth; k++) if (stk[k] == "pf") inarm = 1
        if (INSTR) { walk(line, 0); next }
        t = trim(line)
        if (GUARDED == "1") {
            if (t == "# prani:real-source") { pending = 1; next }
            if (pending) { pending = 0; inguard = 1; next }   # the guard #ifdef itself
            if (t == "# prani:real-source-end") { inguard = 0; ending = 1; next }
            if (ending) { ending = 0; next }
        }
        if (substr(t, 1, 1) == "#") {
            dir_words(t)
            if (DW1 == "#if" || DW1 == "#ifdef" || DW1 == "#ifndef") {
                stk[++depth] = (DW2 == "PRANI_F32") ? "pf" : "other"
            } else if (DW1 == "#endif" && depth > 0) depth--
            next
        }
        walk(line, 0)
        if (!inguard || inarm) next
        if (WCOM ~ /f64-ok:/) next
        s = " " WNOSTR
        why = ""
        if (s ~ /[^A-Za-z0-9_]f64_[a-z0-9_]+[ \t]*\(/) why = "an f64_* call"
        else if (s ~ /[^A-Za-z0-9_]F64_[A-Z0-9_]+/) why = "an F64_* constant"
        else {
            r = s
            while (match(r, /0x[0-9A-Fa-f]+/)) {
                if (RLENGTH >= 18 && substr(r, RSTART - 8, 8) != "pr_of64(") { why = "a raw 16-digit hex literal"; break }
                r = substr(r, RSTART + RLENGTH)
            }
        }
        if (why != "") { printf "%s:%d: %s in code the f32 build compiles: %s\n", SRC, FNR, why, trim(line); bad = 1 }
    }
    END { exit bad }' "$1"
}

fail=0
n=0
for f in src/*.cyr; do
    [ "$f" = "src/real.cyr" ] && continue   # the format layer itself
    if grep -qx '# prani:real-source' "$f"; then
        gen "$f" > "$out/$(basename "$f")" || fail=1
        n=$((n + 1))
        lint "$f" 1 || fail=1
    else
        lint "$f" 0 || fail=1
    fi
done
if [ $fail -ne 0 ]; then
    echo "gen-f64: refused (see above)" >&2
    exit 1
fi

# [lib].modules: every real source immediately followed by its twin, and no twin
# without its source (audit A2).
mods=$(awk '/^\[lib\]/ { on = 1; next } on && /^\[/ { exit } on' cyrius.cyml |
    grep -o '"src/[A-Za-z0-9_/]*\.cyr"' | tr -d '"')
prev=""
for m in $mods; do
    case "$m" in
        src/f64/*)
            want="src/$(basename "$m")"
            if [ "$prev" != "$want" ] || ! grep -qx '# prani:real-source' "$want"; then
                echo "gen-f64: cyrius.cyml lists $m without its source $want just before it" >&2
                fail=1
            fi ;;
        *)
            if [ -f "$m" ] && grep -qx '# prani:real-source' "$m"; then
                nxt=$(echo "$mods" | awk -v m="$m" 'f { print; exit } $0 == m { f = 1 }')
                if [ "$nxt" != "src/f64/$(basename "$m")" ]; then
                    echo "gen-f64: cyrius.cyml lists $m without its twin src/f64/$(basename "$m") right after it" >&2
                    fail=1
                fi
            fi ;;
    esac
    prev="$m"
done
for f in src/*.cyr; do
    if grep -qx '# prani:real-source' "$f" && ! echo "$mods" | grep -qx "$f"; then
        echo "gen-f64: $f is a real source that cyrius.cyml [lib].modules does not list" >&2
        fail=1
    fi
done
if [ $fail -ne 0 ]; then
    exit 1
fi

long=$(awk 'length > 120 { print FILENAME ":" FNR }' "$out"/*.cyr)
if [ -n "$long" ]; then
    echo "gen-f64: lines over 120 characters in the generated f64 build:" >&2
    echo "$long" >&2
    exit 1
fi

if [ $check -eq 1 ]; then
    if ! diff -r "$out" src/f64 >/dev/null 2>&1; then
        diff -r "$out" src/f64 | head -20 >&2 || true
        echo "gen-f64: src/f64/ is stale -- run 'sh scripts/gen-f64.sh'." >&2
        exit 1
    fi
    echo "gen-f64: src/f64/ is current ($n modules); [lib].modules pairs them; the format lint is clean"
else
    echo "gen-f64: wrote $n modules to src/f64/"
fi
