#!/bin/bash
# Build Hexorder.app macOS bundle.
# Usage: ./build/macos/bundle.sh [--release] [--universal]
#
# Options:
#   --release    Build with release optimizations
#   --universal  Build a universal binary (Intel + Apple Silicon)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"

PROFILE="debug"
UNIVERSAL=false

for arg in "$@"; do
    case "$arg" in
        --release) PROFILE="release" ;;
        --universal) UNIVERSAL=true ;;
    esac
done

# --- Build ---

if $UNIVERSAL; then
    echo "Building universal binary (x86_64 + aarch64)..."
    CARGO_FLAGS=""
    if [[ "$PROFILE" == "release" ]]; then
        CARGO_FLAGS="--release"
    fi
    cargo build $CARGO_FLAGS --target x86_64-apple-darwin
    cargo build $CARGO_FLAGS --target aarch64-apple-darwin

    INTEL_BIN="$PROJECT_DIR/target/x86_64-apple-darwin/$PROFILE/hexorder"
    ARM_BIN="$PROJECT_DIR/target/aarch64-apple-darwin/$PROFILE/hexorder"
else
    if [[ "$PROFILE" == "release" ]]; then
        cargo build --release
    else
        cargo build
    fi
fi

# --- Bundle structure ---

BUNDLE_DIR="$PROJECT_DIR/target/$PROFILE/Hexorder.app"
CONTENTS_DIR="$BUNDLE_DIR/Contents"
MACOS_DIR="$CONTENTS_DIR/MacOS"
RESOURCES_DIR="$CONTENTS_DIR/Resources"

# Clean previous bundle.
rm -rf "$BUNDLE_DIR"

# Create bundle structure.
mkdir -p "$MACOS_DIR"
mkdir -p "$RESOURCES_DIR"

# --- Copy binary ---

if $UNIVERSAL; then
    lipo "$INTEL_BIN" "$ARM_BIN" -create -output "$MACOS_DIR/hexorder"
    echo "  Universal binary created"
else
    cp "$PROJECT_DIR/target/$PROFILE/hexorder" "$MACOS_DIR/hexorder"
fi

# --- Copy Info.plist ---

cp "$SCRIPT_DIR/Info.plist" "$CONTENTS_DIR/Info.plist"

# --- Copy icon ---

cp "$PROJECT_DIR/assets/icon/hexorder.icns" "$RESOURCES_DIR/hexorder.icns"

# --- Copy Bevy assets ---
# Bevy looks for assets/ next to the executable (Contents/MacOS/),
# NOT in Contents/Resources/. This is a Bevy-specific requirement.

if [[ -d "$PROJECT_DIR/assets" ]]; then
    # Copy everything except the icon build artifacts (only needed at build time).
    rsync -a --exclude='icon/' "$PROJECT_DIR/assets/" "$MACOS_DIR/assets/"
    echo "  Assets copied to Contents/MacOS/assets/"
fi

echo "Built: $BUNDLE_DIR"
echo "Run with: open $BUNDLE_DIR"
