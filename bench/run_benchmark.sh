#!/bin/bash

# =============================================================================
# sview-fmindex Benchmark Script
# Reproduces the benchmarks shown in README.md
#
# Requirements:
#   - sudo (for drop_caches)
#   - /usr/bin/time (for memory measurement)
#
# Usage:
#   cd bench
#   cargo build --release
#   sudo ./run_benchmark.sh
# =============================================================================

set -e

if [ "$EUID" -ne 0 ]; then
    echo "Please run as root: sudo ./run_benchmark.sh"
    exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

BENCH_BIN="$SCRIPT_DIR/../target/release/sview-fmindex-bench"
DATA_DIR="$SCRIPT_DIR/benchmark_data"
RESULTS_FILE="$SCRIPT_DIR/benchmark_results.csv"

# Check if binary exists
if [ ! -f "$BENCH_BIN" ]; then
    echo "Binary not found. Building..."
    cd "$SCRIPT_DIR" && cargo build --release
fi

# Test parameters (same as README)
TEXT_SIZE=1000000000  # 1G
PATTERN_COUNTS=(10 1000 100000)
COLD_RATIOS=(0.01 0.1 1.0)
PATTERN_LENGTH=20

mkdir -p "$DATA_DIR"

echo "=============================================="
echo "sview-fmindex Benchmark"
echo "=============================================="
echo ""

# Initialize results
echo "pattern_count,cold_ratio,algorithm,total_ns,load_percent,max_rss_kb" > "$RESULTS_FILE"

drop_caches() {
    sync
    echo 3 > /proc/sys/vm/drop_caches
}

# Generate text
echo "Generating 1G text..."
$BENCH_BIN generate-text --data-dir "$DATA_DIR" --text-length $TEXT_SIZE --seed 42 --overwrite

# Build indices
echo "Building indices..."
for ALGO in sview-memory sview-mmap; do
    echo "  $ALGO..."
    $BENCH_BIN build --data-dir "$DATA_DIR" --algorithm $ALGO --sasr 2 --klts 3 > /dev/null 2>&1
done

echo ""
echo "Running benchmarks..."
echo ""

for COLD_RATIO in "${COLD_RATIOS[@]}"; do
    COLD_PCT=$(echo "$COLD_RATIO * 100" | bc | cut -d. -f1)
    echo "=== Cold = ${COLD_PCT}% ==="
    echo "| Patterns | Blob | Elapsed (s) | Index Load (%) | Max RSS |"
    echo "|----------|------|-------------|----------------|---------|"

    for PATTERN_COUNT in "${PATTERN_COUNTS[@]}"; do
        # Generate patterns
        $BENCH_BIN generate-pattern --data-dir "$DATA_DIR" \
            --pattern-length $PATTERN_LENGTH \
            --pattern-count $PATTERN_COUNT \
            --cold-ratio $COLD_RATIO \
            --seed 42 --overwrite > /dev/null 2>&1

        for ALGO in sview-memory sview-mmap; do
            drop_caches

            # Run with memory measurement
            LOG=$(mktemp)
            TIME_OUT=$(/usr/bin/time -v $BENCH_BIN locate --data-dir "$DATA_DIR" --algorithm $ALGO 2>&1 | tee "$LOG")

            # Parse results
            BLOB_LOAD=$(grep "Blob loading time:" "$LOG" | sed 's/.*: //' | sed 's/ ns//')
            TOTAL=$(grep "Total time:" "$LOG" | tail -1 | sed 's/.*: //' | sed 's/ ns//')
            MAX_RSS=$(echo "$TIME_OUT" | grep "Maximum resident set size" | awk '{print $NF}')

            # Calculate load percent
            if [ -n "$BLOB_LOAD" ] && [ -n "$TOTAL" ] && [ "$TOTAL" -gt 0 ]; then
                LOAD_PCT=$(echo "scale=0; $BLOB_LOAD * 100 / $TOTAL" | bc)
            else
                LOAD_PCT=0
            fi

            # Format output
            TOTAL_SEC=$(echo "scale=2; $TOTAL / 1000000000" | bc)
            RSS_MB=$(echo "scale=0; $MAX_RSS / 1024" | bc)

            if [ $RSS_MB -gt 1024 ]; then
                RSS_FMT="$(echo "scale=2; $MAX_RSS / 1024 / 1024" | bc) GiB"
            else
                RSS_FMT="${RSS_MB} MB"
            fi

            # Format blob name
            if [ "$ALGO" = "sview-memory" ]; then
                BLOB_NAME="in-memory"
            else
                BLOB_NAME="mmap"
            fi

            if [ "$PATTERN_COUNT" -eq "${PATTERN_COUNTS[0]}" ]; then
                echo "| $PATTERN_COUNT | $BLOB_NAME | ${TOTAL_SEC} | ${LOAD_PCT} | $RSS_FMT |"
            else
                echo "| | $BLOB_NAME | ${TOTAL_SEC} | ${LOAD_PCT} | $RSS_FMT |"
            fi

            # Save to CSV
            echo "$PATTERN_COUNT,$COLD_RATIO,$ALGO,$TOTAL,$LOAD_PCT,$MAX_RSS" >> "$RESULTS_FILE"

            rm -f "$LOG"
        done
    done
    echo ""
done

echo "Results saved to: $RESULTS_FILE"
echo "Data directory: $DATA_DIR"
