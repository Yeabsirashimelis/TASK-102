#!/bin/bash
##############################################################################
# Unit Test Runner
# Runs Rust unit tests inside a Docker container (requires libpq for linking).
# Usage: bash unit_tests/run_unit_tests.sh
##############################################################################

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_DIR"

echo "============================================"
echo "  RetailOps Unit Tests"
echo "============================================"
echo ""

# Resolve absolute path for Docker volume mount (Windows-compatible)
PROJ_PATH="$(cd "$PROJECT_DIR" && pwd -W 2>/dev/null || pwd)"

echo "[INFO] Building and running unit tests in Docker..."
export MSYS_NO_PATHCONV=1
docker run --rm \
  -v "${PROJ_PATH}:/app" \
  -w /app \
  rust:1.88-bookworm \
  bash -c "
    apt-get update -qq && apt-get install -y -qq libpq-dev > /dev/null 2>&1
    echo '[INFO] Running cargo test...'
    cargo test --release 2>&1
    EXIT_CODE=\$?
    echo ''
    if [ \$EXIT_CODE -eq 0 ]; then
      echo '============================================'
      echo '  UNIT TESTS: ALL PASSED'
      echo '============================================'
    else
      echo '============================================'
      echo '  UNIT TESTS: SOME FAILED'
      echo '============================================'
    fi
    exit \$EXIT_CODE
  "
