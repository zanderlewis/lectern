#!/bin/bash
set -e

# Lectern vs Composer Benchmark using Hyperfine
# This script uses hyperfine for accurate performance measurements with cache warm-up

REPORT_FILE="BENCHMARK.md"

echo "🔨 Building Lectern in release mode..."
cargo build --release --quiet

LECTERN="$(pwd)/target/release/lectern"
COMPOSER="composer"
BENCH_DIR="$(pwd)/bench-test"

echo ""
echo "=================================================="
echo "  Lectern vs Composer Performance Benchmark"
echo "=================================================="
echo ""

# Check if hyperfine is installed
if ! command -v hyperfine &> /dev/null; then
    echo "❌ hyperfine is not installed!"
    echo "Install it with: cargo install hyperfine"
    exit 1
fi

# Create a test project directory
echo "📦 Setting up test project..."
rm -rf "$BENCH_DIR"
mkdir -p "$BENCH_DIR"
cd "$BENCH_DIR"

# Create a simple composer.json
cat > composer.json << 'EOF'
{
    "name": "bench/test",
    "description": "Benchmark test project",
    "type": "project",
    "license": ["MIT"],
    "require": {
        "monolog/monolog": "^3.0",
        "guzzlehttp/guzzle": "^7.2"
    },
    "require-dev": {
        "phpunit/phpunit": "^10.0"
    },
    "autoload": {
        "psr-4": {
            "App\\": "src/"
        }
    },
    "minimum-stability": "stable",
    "prefer-stable": true
}
EOF

# Do initial install to populate vendor and lock files
echo "📥 Initial setup (installing packages)..."
if ! $LECTERN install > /dev/null 2>&1; then
    echo "❌ Failed to install packages with lectern"
    echo "Running with output for debugging:"
    $LECTERN install
    exit 1
fi

# Initialize markdown report
cd ..
cat > "$REPORT_FILE" << 'EOF'
# Lectern vs Composer Performance Benchmark

> Generated on: DATE_PLACEHOLDER

This benchmark compares the performance of Lectern against Composer across various common operations.

## System Information

- **Lectern Version**: `lectern --version` output
- **Composer Version**: `composer --version` output
- **Benchmark Tool**: hyperfine with 2 warmup runs and 5 test runs (3 for require/remove)

## Results Summary

| Command | Lectern | Composer | Speedup |
|---------|---------|----------|---------|
EOF

# Replace date placeholder
sed -i "s/DATE_PLACEHOLDER/$(date '+%Y-%m-%d %H:%M:%S')/" "$REPORT_FILE"

cd "$BENCH_DIR"

echo ""
echo "Starting benchmarks with warm caches..."
echo ""

# Install command
echo "📦 Benchmarking: install"
hyperfine --warmup 2 --runs 5 --export-markdown /tmp/bench_install.md \
    --prepare "rm -rf vendor composer.lock" \
    --command-name "lectern install" "$LECTERN --quiet install" \
    --command-name "composer install" "$COMPOSER install --quiet --no-interaction"

# Update command
echo ""
echo "📦 Benchmarking: update"
hyperfine --warmup 2 --runs 5 --export-markdown /tmp/bench_update.md \
    --command-name "lectern update" "$LECTERN --quiet update" \
    --command-name "composer update" "$COMPOSER update --quiet --no-interaction"

# Search command
echo ""
echo "🔍 Benchmarking: search"
hyperfine --warmup 2 --runs 5 --export-markdown /tmp/bench_search.md \
    --command-name "lectern search" "$LECTERN --quiet search monolog" \
    --command-name "composer search" "$COMPOSER search monolog --quiet"

# Show command
echo ""
echo "📄 Benchmarking: show"
hyperfine --warmup 2 --runs 5 --export-markdown /tmp/bench_show.md \
    --command-name "lectern show" "$LECTERN --quiet show monolog/monolog" \
    --command-name "composer show" "$COMPOSER show monolog/monolog --quiet"

# Outdated command
echo ""
echo "🔍 Benchmarking: outdated"
hyperfine --warmup 2 --runs 5 --export-markdown /tmp/bench_outdated.md \
    --command-name "lectern outdated" "$LECTERN --quiet outdated" \
    --command-name "composer outdated" "$COMPOSER outdated --quiet"

# Licenses command
echo ""
echo "📜 Benchmarking: licenses"
hyperfine --warmup 2 --runs 5 --export-markdown /tmp/bench_licenses.md \
    --command-name "lectern licenses" "$LECTERN --quiet licenses" \
    --command-name "composer licenses" "$COMPOSER licenses --quiet"

# Status command
echo ""
echo "📊 Benchmarking: status"
hyperfine --warmup 2 --runs 5 --export-markdown /tmp/bench_status.md \
    --command-name "lectern status" "$LECTERN --quiet status" \
    --command-name "composer show" "$COMPOSER show --quiet"

# Require command
echo ""
echo "➕ Benchmarking: require (add package)"
# Backup original files
cp composer.json composer.json.orig
cp composer.lock composer.lock.orig

hyperfine --warmup 1 --runs 3 --export-markdown /tmp/bench_require.md \
    --prepare "cp composer.json.orig composer.json; cp composer.lock.orig composer.lock" \
    --command-name "lectern require" "$LECTERN --quiet require symfony/yaml:^6.0" \
    --command-name "composer require" "$COMPOSER require symfony/yaml:^6.0 --quiet --no-interaction"

# Remove command - first add the package to both so we have a consistent starting state
echo ""
echo "➖ Benchmarking: remove (remove package)"
# Create versions with symfony/yaml added by each tool
cp composer.json.orig composer.json
cp composer.lock.orig composer.lock
$LECTERN --quiet require symfony/yaml:^6.0 > /dev/null 2>&1
cp composer.json composer.json.lectern
cp composer.lock composer.lock.lectern

cp composer.json.orig composer.json
cp composer.lock.orig composer.lock
$COMPOSER require symfony/yaml:^6.0 --quiet --no-interaction > /dev/null 2>&1
cp composer.json composer.json.composer
cp composer.lock composer.lock.composer

hyperfine --warmup 1 --runs 3 --export-markdown /tmp/bench_remove.md \
    --prepare "cp composer.json.lectern composer.json; cp composer.lock.lectern composer.lock" \
    --command-name "lectern remove" "$LECTERN --quiet remove symfony/yaml" \
    --prepare "cp composer.json.composer composer.json; cp composer.lock.composer composer.lock" \
    --command-name "composer remove" "$COMPOSER remove symfony/yaml --quiet --no-interaction"

# Cleanup
cd ..
rm -rf "$BENCH_DIR"

# Generate the markdown report
echo ""
echo "📊 Generating report..."

# Function to extract times and calculate speedup
process_benchmark() {
    local file=$1
    local name=$2
    local lectern_time=$(grep "lectern" "$file" | awk '{print $3}')
    local composer_time=$(grep "composer" "$file" | awk '{print $3}')
    
    # Calculate speedup using bc if available, otherwise use awk
    if command -v bc &> /dev/null; then
        local speedup=$(echo "scale=2; $composer_time / $lectern_time" | bc)
    else
        local speedup=$(awk "BEGIN {printf \"%.2f\", $composer_time / $lectern_time}")
    fi
    
    echo "| $name | ${lectern_time} | ${composer_time} | ${speedup}x |" >> "$REPORT_FILE"
}

process_benchmark "/tmp/bench_install.md" "install"
process_benchmark "/tmp/bench_update.md" "update"
process_benchmark "/tmp/bench_search.md" "search"
process_benchmark "/tmp/bench_show.md" "show"
process_benchmark "/tmp/bench_outdated.md" "outdated"
process_benchmark "/tmp/bench_licenses.md" "licenses"
process_benchmark "/tmp/bench_status.md" "status"
process_benchmark "/tmp/bench_require.md" "require"
process_benchmark "/tmp/bench_remove.md" "remove"

# Add detailed results section
cat >> "$REPORT_FILE" << 'EOF'

## Detailed Results

### Install Command
EOF
tail -n +3 /tmp/bench_install.md >> "$REPORT_FILE"

cat >> "$REPORT_FILE" << 'EOF'

### Update Command
EOF
tail -n +3 /tmp/bench_update.md >> "$REPORT_FILE"

cat >> "$REPORT_FILE" << 'EOF'

### Search Command
EOF
tail -n +3 /tmp/bench_search.md >> "$REPORT_FILE"

cat >> "$REPORT_FILE" << 'EOF'

### Show Command
EOF
tail -n +3 /tmp/bench_show.md >> "$REPORT_FILE"

cat >> "$REPORT_FILE" << 'EOF'

### Outdated Command
EOF
tail -n +3 /tmp/bench_outdated.md >> "$REPORT_FILE"

cat >> "$REPORT_FILE" << 'EOF'

### Licenses Command
EOF
tail -n +3 /tmp/bench_licenses.md >> "$REPORT_FILE"

cat >> "$REPORT_FILE" << 'EOF'

### Status Command
EOF
tail -n +3 /tmp/bench_status.md >> "$REPORT_FILE"

cat >> "$REPORT_FILE" << 'EOF'

### Require Command
EOF
tail -n +3 /tmp/bench_require.md >> "$REPORT_FILE"

cat >> "$REPORT_FILE" << 'EOF'

### Remove Command
EOF
tail -n +3 /tmp/bench_remove.md >> "$REPORT_FILE"

# Cleanup temp files
rm -f /tmp/bench_*.md

echo ""
echo "=================================================="
echo "  ✅ Benchmark Complete!"
echo "=================================================="
echo "📄 Report saved to: $REPORT_FILE"
echo ""

