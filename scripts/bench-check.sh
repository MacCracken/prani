#!/bin/sh
# bench-check.sh -- run the benchmark harness and compare it against a committed
# baseline, printing a delta per benchmark.
#
# 2.0.11. `cyrius audit` runs the bench harness and reports "1 passed, 0 failed"
# because all it checks is that the harness RAN -- it compares nothing. Three
# regressions reached a release through that gap (see tests/allocbudget.tcyr).
# This is the timing half of the fix; the allocation half is that suite.
#
# ⚠ REPORT-ONLY BY DEFAULT, DELIBERATELY. Wall-clock on a shared CI runner is
# noisy -- a 20% swing between runs on an unchanged tree is ordinary -- and a
# gate that cries wolf gets disabled within a month, which is worse than no
# gate. So this prints and exits 0. Allocation, which IS deterministic, is what
# gets hard-gated.
#
#   BENCH_GATE=1.5 sh scripts/bench-check.sh    # opt in: fail past 1.5x
#
# Turn the gate on only once recorded history shows this runner is quiet enough
# to justify it. Until then the delta table is for a human to read.
#
# POSIX sh -- CI runs this under dash, where `set -o pipefail` is an error.
set -eu

BASELINE="${BASELINE:-benches/baseline.csv}"
BENCH_FILE="${BENCH_FILE:-tests/prani.bcyr}"
GATE="${BENCH_GATE:-0}"

command -v cyrius >/dev/null 2>&1 || { echo "error: cyrius not on PATH" >&2; exit 1; }
[ -f "$BASELINE" ] || { echo "error: no baseline at $BASELINE" >&2; exit 1; }

# The harness benchmarks dist/prani.cyr -- the generated bundle, deliberately,
# since that is what a consumer gets. So a STALE bundle measures the last
# bundled code, not this tree. 2.0.10 lost half a day to exactly that: the audit
# read 155 ns for a change that measured 114. Warn loudly.
if [ -f dist/prani.cyr ]; then
    for f in src/*.cyr; do
        if [ "$f" -nt dist/prani.cyr ]; then
            echo "⚠ WARNING: $f is newer than dist/prani.cyr." >&2
            echo "  The harness measures the BUNDLE. Run 'cyrius distlib' first," >&2
            echo "  or these numbers describe the last bundled code, not this tree." >&2
            break
        fi
    done
fi

LOCK="${CYRIUS_LOCKFILE:-${TMPDIR:-/tmp}/prani-build.lock}"
OUT="${TMPDIR:-/tmp}/prani-bench-check.$$"
if command -v flock >/dev/null 2>&1; then
    flock "$LOCK" cyrius bench "$BENCH_FILE" > "$OUT" 2>&1
else
    cyrius bench "$BENCH_FILE" > "$OUT" 2>&1
fi

awk -v baseline="$BASELINE" -v gate="$GATE" '
function to_ns(v, u) {
    if (u == "ns") return v
    if (u == "us") return v * 1000
    if (u == "ms") return v * 1000000
    if (u == "s")  return v * 1000000000
    return -1
}
BEGIN {
    FS = ","
    n = 0
    while ((getline line < baseline) > 0) {
        if (line ~ /^#/ || line ~ /^benchmark,/) continue
        split(line, f, ",")
        name = f[1]; gsub(/^"|"$/, "", name)
        base[name] = f[2] + 0
        order[n++] = name
    }
    close(baseline)
    printf "%-42s %12s %12s %9s\n", "benchmark", "baseline", "now", "delta"
    printf "%-42s %12s %12s %9s\n", "------------------------------------------", "------------", "------------", "---------"
    worst = 1.0; worst_name = "-"
}
{
    line = $0
    if (match(line, /^[ \t]*[^:]+: [0-9.]+(ns|us|ms|s) avg/) == 0) next
    nm = line; sub(/^[ \t]+/, "", nm); sub(/:.*/, "", nm)
    val = line; sub(/^[^:]*: /, "", val); sub(/ avg.*/, "", val)
    unit = val; gsub(/[0-9.]/, "", unit)
    num = val + 0
    cur[nm] = to_ns(num, unit)
    seen[nm] = 1
}
END {
    missing = 0
    for (i = 0; i < n; i++) {
        nm = order[i]
        if (!(nm in seen)) { printf "%-42s %12.1f %12s %9s\n", nm, base[nm], "MISSING", "!"; missing++; continue }
        r = cur[nm] / base[nm]
        printf "%-42s %12.1f %12.1f %8.2fx\n", nm, base[nm], cur[nm], r
        if (r > worst) { worst = r; worst_name = nm }
    }
    for (nm in seen) if (!(nm in base)) printf "%-42s %12s %12.1f %9s\n", nm, "(new)", cur[nm], "-"
    printf "\nworst regression: %.2fx (%s)\n", worst, worst_name
    if (missing > 0) printf "⚠ %d baseline benchmark(s) did not run\n", missing
    if (gate + 0 > 0) {
        if (worst > gate + 0) { printf "FAIL: %.2fx exceeds BENCH_GATE=%.2fx\n", worst, gate + 0; exit 1 }
        printf "ok: within BENCH_GATE=%.2fx\n", gate + 0
    } else {
        printf "(report-only; set BENCH_GATE=<factor> to fail on regression)\n"
    }
    if (missing > 0) exit 1
}
' "$OUT"
rc=$?
rm -f "$OUT"
exit $rc
