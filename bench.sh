#!/bin/bash

# Basic shell script for benchmarking

echo "Starting benchmark..."

start_time=$(date +%s)

cwd=$(pwd)

cd "$cwd/benchmarks" || exit 1
uv run benchmark.py -o performance-report.md || exit 1
cd "$cwd" || exit 1

end_time=$(date +%s)
elapsed=$((end_time - start_time))

echo "Benchmark completed in $elapsed seconds."