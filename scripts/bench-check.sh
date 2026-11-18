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
# RE-RECORDING. A baseline is a claim about one host AND one toolchain. Through
# 2.2.0 the only way to make one was by hand, and the committed one was cyrius
# 6.5.36's: on 6.7.18 16 of 18 rows read 0.50-0.84x, so a 2.4x regression in
# wolf_howl_1s would have read 1.48x and passed a 1.5x gate (audit D5, 2026-11-17).
#
#   sh scripts/bench-check.sh --write    # record benches/baseline.csv
#
# runs the harness BENCH_RUNS times (default 3) and records each benchmark's
# minimum, with the host, the toolchain and the tree in the header. A compare
# warns when the baseline's toolchain is not the one running. Re-record after
# every toolchain or dependency bump, in a commit of its own.
#
# POSIX sh -- CI runs this under dash, where `set -o pipefail` is an error.
set -eu

BASELINE="${BASELINE:-benches/baseline.csv}"
BENCH_FILE="${BENCH_FILE:-tests/prani.bcyr}"
GATE="${BENCH_GATE:-0}"
MODE=compare
case "${1:-}" in
    --write) MODE=write ;;
    "") ;;
    *) echo "usage: sh scripts/bench-check.sh [--write]" >&2; exit 2 ;;
esac
if [ "$MODE" = write ]; then RUNS="${BENCH_RUNS:-3}"; else RUNS="${BENCH_RUNS:-1}"; fi

command -v cyrius >/dev/null 2>&1 || { echo "error: cyrius not on PATH" >&2; exit 1; }
if [ "$MODE" = compare ]; then
    [ -f "$BASELINE" ] || { echo "error: no baseline at $BASELINE (record one with --write)" >&2; exit 1; }
fi
TOOLCHAIN=$(cyrius --version 2>/dev/null | head -n 1)

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
RUN="$OUT.run"
# Through 2.2.0 a failing harness ended the script under `set -e` before its
# output was printed or removed: rc=1, no diagnostic, a file left in TMPDIR
# (audit D12). Now the output is shown and the files go on every exit.
trap 'rm -f "$OUT" "$RUN"' EXIT INT TERM
bench_once() {
    if command -v flock >/dev/null 2>&1; then
        flock "$LOCK" cyrius bench "$BENCH_FILE"
    else
        cyrius bench "$BENCH_FILE"
    fi
}
: > "$OUT"
i=0
while [ "$i" -lt "$RUNS" ]; do
    bench_once > "$RUN" 2>&1 || {
        echo "error: cyrius bench $BENCH_FILE failed:" >&2
        cat "$RUN" >&2
        exit 1
    }
    cat "$RUN" >> "$OUT"
    i=$((i + 1))
done

# The harness's result lines: "  name: 123.4ns avg ...". Each becomes
# "name<TAB>nanoseconds", the minimum across the runs, in first-seen order.
results() {
    awk '
    function to_ns(v, u) {
        if (u == "ns") return v
        if (u == "us") return v * 1000
        if (u == "ms") return v * 1000000
        if (u == "s")  return v * 1000000000
        return -1
    }
    {
        line = $0
        if (match(line, /^[ \t]*[^:]+: [0-9.]+(ns|us|ms|s) avg/) == 0) next
        nm = line; sub(/^[ \t]+/, "", nm); sub(/:.*/, "", nm)
        val = line; sub(/^[^:]*: /, "", val); sub(/ avg.*/, "", val)
        unit = val; gsub(/[0-9.]/, "", unit)
        ns = to_ns(val + 0, unit)
        if (!(nm in best)) { order[n++] = nm; best[nm] = ns }
        else if (ns < best[nm]) best[nm] = ns
    }
    END { for (i = 0; i < n; i++) printf "%s\t%.1f\n", order[i], best[order[i]] }
    ' "$OUT"
}

if [ "$MODE" = write ]; then
    host="$(uname -s) $(uname -m)"
    cpu=$(sed -n 's/^model name[[:space:]]*: //p' /proc/cpuinfo 2>/dev/null | head -n 1)
    [ -n "$cpu" ] && host="$host, $cpu"
    tree="prani $(cat VERSION 2>/dev/null || echo '?')"
    rev=$(git rev-parse --short HEAD 2>/dev/null || true)
    if [ -n "$rev" ]; then
        if git diff --quiet HEAD -- 2>/dev/null; then tree="$tree at $rev"; else tree="$tree at $rev + uncommitted changes"; fi
    fi
    n=$(results | wc -l | tr -d ' ')
    [ "$n" -gt 0 ] || { echo "error: the harness printed no results" >&2; cat "$OUT" >&2; exit 1; }
    {
        if [ "$RUNS" -eq 1 ]; then runs_said="from one run"; else runs_said="each the fastest of $RUNS runs"; fi
        echo "# prani benchmark baseline -- nanoseconds, $runs_said."
        echo "# Re-record with \`sh scripts/bench-check.sh --write\` after every toolchain or"
        echo "# dependency bump, in a commit of its own. A number here is a claim about THIS"
        echo "# host and THIS toolchain; a different one will differ, and that is not a"
        echo "# regression."
        echo "# host: $host"
        echo "# toolchain: $TOOLCHAIN"
        echo "# recorded: $(date -u +%Y-%m-%d), $tree"
        echo "benchmark,baseline_ns"
        results | awk -F '\t' '{ printf "\"%s\",%s\n", $1, $2 }'
    } > "$BASELINE.new"
    mv "$BASELINE.new" "$BASELINE"
    echo "bench-check: recorded $n benchmarks to $BASELINE ($TOOLCHAIN, $RUNS runs)"
    exit 0
fi

recorded=$(sed -n 's/^# toolchain: //p' "$BASELINE" | head -n 1)
if [ "$recorded" != "$TOOLCHAIN" ]; then
    echo "⚠ WARNING: $BASELINE was recorded on '${recorded:-an unrecorded toolchain}'," >&2
    echo "  and this is '$TOOLCHAIN'. The deltas below compare two compilers." >&2
    echo "  Re-record with: sh scripts/bench-check.sh --write" >&2
fi

results | awk -v baseline="$BASELINE" -v gate="$GATE" '
BEGIN {
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
    split($0, fld, "\t")
    cur[fld[1]] = fld[2] + 0
    seen[fld[1]] = 1
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
'
