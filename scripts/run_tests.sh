#!/usr/bin/env bash
# scripts/run_tests.sh
# Run the full barq-vault test suite across all workspace crates.
# Usage: ./scripts/run_tests.sh [--verbose]

set -euo pipefail

VERBOSE=${1:-""}
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

echo "================================================"
echo "  barq-vault — Full Test Suite"
echo "================================================"
echo ""

CARGO_FLAGS="--workspace --exclude barq-server"
if [[ "$VERBOSE" == "--verbose" || "$VERBOSE" == "-v" ]]; then
    CARGO_FLAGS="$CARGO_FLAGS -- --nocapture"
fi

echo "[1/5] Checking workspace compiles..."
cargo check --workspace 2>&1
echo "      ✓ cargo check passed"
echo ""

echo "[2/5] Running barq-types tests..."
cargo test -p barq-types 2>&1 | tail -3
echo ""

echo "[3/5] Running barq-compress tests..."
cargo test -p barq-compress 2>&1 | tail -3
echo ""

echo "[4/5] Running barq-index tests..."
cargo test -p barq-index 2>&1 | tail -3
echo ""

echo "[5/5] Running barq-ingest, barq-proto, barq-store, barq-server tests..."
cargo test -p barq-store -p barq-ingest -p barq-proto -p barq-client 2>&1 | grep "^test result"
echo ""

echo "================================================"
echo "  Running barq-server integration tests..."
echo "================================================"
cargo test -p barq-server 2>&1 | grep "test result"
echo ""

echo "================================================"
echo "  ALL TESTS COMPLETE"
echo "================================================"
