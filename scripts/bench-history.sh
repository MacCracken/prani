#!/usr/bin/env bash
# bench-history.sh — Run benchmarks and append results to history log
set -euo pipefail

HISTORY_FILE="benches/history.csv"
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
GIT_REV=$(git rev-parse --short HEAD 2>/dev/null || echo "uncommitted")

# Create header if file doesn't exist
if [ ! -f "$HISTORY_FILE" ]; then
    echo "timestamp,git_rev,benchmark,time_us" > "$HISTORY_FILE"
fi

# Run benchmarks and parse output
cargo bench 2>&1 | grep -E "^\S.*time:" | while IFS= read -r line; do
    bench_name=$(echo "$line" | awk '{print $1}')
    # Extract the middle (estimate) value in microseconds
    time_val=$(echo "$line" | grep -oP '\[\K[0-9.]+ [µmn]s' | head -2 | tail -1)
    # Normalize to microseconds
    num=$(echo "$time_val" | awk '{print $1}')
    unit=$(echo "$time_val" | awk '{print $2}')
    case "$unit" in
        "ns") num=$(echo "$num / 1000" | bc -l) ;;
        "ms") num=$(echo "$num * 1000" | bc -l) ;;
        "µs") ;; # already in µs
    esac
    echo "${TIMESTAMP},${GIT_REV},${bench_name},${num}" >> "$HISTORY_FILE"
done

echo "Benchmarks recorded to $HISTORY_FILE"
cat "$HISTORY_FILE" | column -t -s,
