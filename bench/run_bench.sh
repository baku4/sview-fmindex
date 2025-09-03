#!/bin/bash

# sview-fmindex 벤치마크 실행 스크립트
# 사용법: ./run_bench.sh [text_length] [pattern_length] [pattern_count]
# 예시: ./run_bench.sh 100000 20 100

set -e

# 기본값 설정
TEXT_LENGTH=${1:-100000}
PATTERN_LENGTH=${2:-20}
PATTERN_COUNT=${3:-100}

echo "🚀 Starting sview-fmindex benchmark"

# 릴리즈 빌드
echo "🔧 Building release version..."
cargo build --release

BENCH_BIN="../target/release/sview-fmindex-bench"

# 1. Generate test data
echo "📊 Generating test data... (text: $TEXT_LENGTH, pattern: $PATTERN_LENGTH, count: $PATTERN_COUNT)"
$BENCH_BIN generate --data-dir test_data --text-length $TEXT_LENGTH --pattern-length $PATTERN_LENGTH --pattern-count $PATTERN_COUNT --seed 42

# 2. Build indices for each algorithm
echo "🔨 Building lt-fm-index..."
$BENCH_BIN build --data-dir test_data --algorithm lt-fm-index --sasr 2 --klts 3 > test_data/build_lt-fm-index.log

echo "🔨 Building sview-memory..."
$BENCH_BIN build --data-dir test_data --algorithm sview-memory --sasr 2 --klts 3 > test_data/build_sview-memory.log

echo "🔨 Building sview-mmap..."
$BENCH_BIN build --data-dir test_data --algorithm sview-mmap --sasr 2 --klts 3 > test_data/build_sview-mmap.log

# 3. Measure search performance for each algorithm
echo "🔍 Testing lt-fm-index search performance..."
$BENCH_BIN locate --data-dir test_data --algorithm lt-fm-index > test_data/locate_lt-fm-index.log

echo "🔍 Testing sview-memory search performance..."
$BENCH_BIN locate --data-dir test_data --algorithm sview-memory > test_data/locate_sview-memory.log

echo "🔍 Testing sview-mmap search performance..."
$BENCH_BIN locate --data-dir test_data --algorithm sview-mmap > test_data/locate_sview-mmap.log

echo "✅ Benchmark completed!"
echo ""
echo "📈 Performance Summary:"
echo "--- Build Performance (Detailed) ---"

# Parse lt-fm-index build performance
echo "lt-fm-index:"
echo "  Build time: $(grep 'Build time:' test_data/build_lt-fm-index.log | sed 's/.*: //')"
echo "  Save time: $(grep 'Save time:' test_data/build_lt-fm-index.log | sed 's/.*: //')"
echo "  Total time: $(grep 'Total time:' test_data/build_lt-fm-index.log | tail -n 1 | sed 's/.*: //')"

echo "sview-memory:"
echo "  Build time: $(grep 'Build time:' test_data/build_sview-memory.log | sed 's/.*: //')"
echo "  Save time: $(grep 'Save time:' test_data/build_sview-memory.log | sed 's/.*: //')"
echo "  Total time: $(grep 'Total time:' test_data/build_sview-memory.log | tail -n 1 | sed 's/.*: //')"

echo "sview-mmap:"
echo "  Build time: $(grep 'Build time:' test_data/build_sview-mmap.log | sed 's/.*: //')"
echo "  Save time: $(grep 'Save time:' test_data/build_sview-mmap.log | sed 's/.*: //')"
echo "  Total time: $(grep 'Total time:' test_data/build_sview-mmap.log | tail -n 1 | sed 's/.*: //')"
echo ""
echo "--- Search Performance (Detailed) ---"

# Parse lt-fm-index search performance
echo "lt-fm-index:"
echo "  Blob loading: $(grep 'Blob loading time:' test_data/locate_lt-fm-index.log | sed 's/.*: //')"
echo "  Locate processing: $(grep 'Locate processing time:' test_data/locate_lt-fm-index.log | sed 's/.*: //')"
echo "  Locate time: $(grep 'Locate time:' test_data/locate_lt-fm-index.log | sed 's/.*: //')"
echo "  Total time: $(grep 'Total time:' test_data/locate_lt-fm-index.log | tail -n 1 | sed 's/.*: //')"

echo "sview-memory:"
echo "  Blob loading: $(grep 'Blob loading time:' test_data/locate_sview-memory.log | sed 's/.*: //')"
echo "  Locate processing: $(grep 'Locate processing time:' test_data/locate_sview-memory.log | sed 's/.*: //')"
echo "  Locate time: $(grep 'Locate time:' test_data/locate_sview-memory.log | sed 's/.*: //')"
echo "  Total time: $(grep 'Total time:' test_data/locate_sview-memory.log | tail -n 1 | sed 's/.*: //')"

echo "sview-mmap:"
echo "  Blob loading: $(grep 'Blob loading time:' test_data/locate_sview-mmap.log | sed 's/.*: //')"
echo "  Locate processing: $(grep 'Locate processing time:' test_data/locate_sview-mmap.log | sed 's/.*: //')"
echo "  Locate time: $(grep 'Locate time:' test_data/locate_sview-mmap.log | sed 's/.*: //')"
echo "  Total time: $(grep 'Total time:' test_data/locate_sview-mmap.log | tail -n 1 | sed 's/.*: //')"