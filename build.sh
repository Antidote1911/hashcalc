#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
BUILD_DIR="$ROOT/gui-qt/build"

# ── 1. Rust workspace (CLI + egui GUI + FFI staticlib) ────────────────────────
echo "==> cargo build --release"
cargo build --release --manifest-path "$ROOT/Cargo.toml"

# ── 2. Qt6 GUI (CMake) ────────────────────────────────────────────────────────
echo "==> cmake configure"
cmake -S "$ROOT/gui-qt" -B "$BUILD_DIR" \
    -DCMAKE_BUILD_TYPE=Release \
    -DCMAKE_PREFIX_PATH="/home/antidote/qt6-static" \
    --fresh 2>&1 | tail -5

echo "==> cmake build"
cmake --build "$BUILD_DIR" --parallel "$(nproc)"

echo ""
echo "Binaries:"
echo "  CLI / egui GUI : $ROOT/target/release/hashcalc"
echo "  Qt6 GUI        : $BUILD_DIR/hashcalc-qt"
