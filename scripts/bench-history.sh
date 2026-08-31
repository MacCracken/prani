#!/usr/bin/env bash
# bench-history.sh — Run the Cyrius benchmark harness and append results to a
# history log, so a performance regression is visible across commits.
#
# Ported from the Rust era (2.0.4): the previous version shelled out to
# `cargo bench` and parsed criterion's output. prani is a Cyrius project —
# `rust-old/benches/` is the frozen oracle's harness, not this project's.
#
# Output: benches/history.csv (gitignored via *.csv — it is a local log, not
# a tracked artifact). Reference numbers live in docs/benchmarks.md.
set -euo pipefail

BENCH_FILE="${BENCH_FILE:-tests/prani.bcyr}"
HISTORY_FILE="${HISTORY_FILE:-benches/history.csv}"
HEADER="timestamp,git_rev,benchmark,avg_us,min_us,max_us,iters,timer_floor_us"

TIMESTAMP="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
GIT_REV="$(git rev-parse --short HEAD 2>/dev/null || echo "uncommitted")"
if ! git diff --quiet HEAD 2>/dev/null; then
    GIT_REV="${GIT_REV}-dirty"
fi

command -v cyrius >/dev/null 2>&1 || {
    echo "error: cyrius not on PATH. See CONTRIBUTING.md — Development Requirements." >&2
    exit 1
}
[ -f "$BENCH_FILE" ] || { echo "error: no bench harness at $BENCH_FILE" >&2; exit 1; }

mkdir -p "$(dirname "$HISTORY_FILE")"
if [ ! -f "$HISTORY_FILE" ]; then
    echo "$HEADER" > "$HISTORY_FILE"
elif [ "$(head -1 "$HISTORY_FILE")" != "$HEADER" ]; then
    echo "error: $HISTORY_FILE has an unrecognised header (pre-2.0.4 format?)." >&2
    echo "       Move it aside and re-run; the schema is: $HEADER" >&2
    exit 1
fi

# Every `cyrius …` call re-resolves deps and races on cyrius.lock — serialize.
LOCK="${CYRIUS_LOCKFILE:-${TMPDIR:-/tmp}/prani-build.lock}"
if command -v flock >/dev/null 2>&1; then
    OUTPUT="$(flock "$LOCK" cyrius bench "$BENCH_FILE" 2>&1)"
else
    OUTPUT="$(cyrius bench "$BENCH_FILE" 2>&1)"
fi
echo "$OUTPUT"

# `cyrius bench` (lib/bench.cyr, cyrius 6.5.19+) prints, per benchmark:
#   <name>: <avg><unit> avg (min=<v><unit> max=<v><unit>) [<n> iters]
# and once per run:
#   [timer floor <v><unit> per clock read, measured; subtracted from every sample]
# Units are ns / us / ms / s; everything is normalised to microseconds here.
#
# sed pulls the fields (portable ERE, `|`-separated — no benchmark name contains
# a pipe); awk does only the unit arithmetic, in POSIX constructs, so this runs
# under mawk (Ubuntu's default awk) as well as gawk.
ROWS="$(
    printf '%s\n' "$OUTPUT" \
    | sed -n -E \
        -e 's/^.*timer floor ([0-9.]+)(ns|us|ms|s) per clock read.*$/FLOOR|\1|\2/p' \
        -e 's/^[[:space:]]*(.+): ([0-9.]+)(ns|us|ms|s) avg \(min=([0-9.]+)(ns|us|ms|s) max=([0-9.]+)(ns|us|ms|s)\) \[([0-9]+) iters\].*$/BENCH|\1|\2|\3|\4|\5|\6|\7|\8/p' \
    | awk -F'|' -v ts="$TIMESTAMP" -v rev="$GIT_REV" '
        function to_us(v, u) {
            if (u == "ns") return v / 1000
            if (u == "us") return v
            if (u == "ms") return v * 1000
            if (u == "s")  return v * 1000000
            return ""
        }
        $1 == "FLOOR" { floor_us = to_us($2 + 0, $3); next }
        $1 == "BENCH" {
            name = $2
            sub(/^[ \t]+/, "", name)
            sub(/[ \t]+$/, "", name)
            gsub(/"/, "\"\"", name)
            printf "%s,%s,\"%s\",%.6f,%.6f,%.6f,%s,%s\n",
                   ts, rev, name,
                   to_us($3 + 0, $4), to_us($5 + 0, $6), to_us($7 + 0, $8),
                   $9, (floor_us == "" ? "" : sprintf("%.6f", floor_us))
        }
    '
)"

if [ -z "$ROWS" ]; then
    echo "error: parsed 0 benchmarks out of $BENCH_FILE — has the bench output format changed?" >&2
    echo "       Nothing was written to $HISTORY_FILE." >&2
    exit 1
fi

printf '%s\n' "$ROWS" >> "$HISTORY_FILE"
echo
echo "Recorded $(printf '%s\n' "$ROWS" | wc -l | tr -d ' ') benchmark(s) to $HISTORY_FILE at $GIT_REV"
