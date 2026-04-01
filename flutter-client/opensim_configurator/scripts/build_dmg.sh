#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

APP_NAME="OpenSim Configurator"
VERSION=$(grep '^version:' "$PROJECT_DIR/pubspec.yaml" | sed 's/version: *//;s/+.*//')
DMG_NAME="OpenSimConfigurator-${VERSION}"
DIST_DIR="$PROJECT_DIR/dist"
BUILD_DIR="$PROJECT_DIR/build/macos/Build/Products/Release"
STAGING_DIR=$(mktemp -d)

cleanup() {
    if [ -d "$STAGING_DIR" ]; then
        rm -rf "$STAGING_DIR"
    fi
}
trap cleanup EXIT

echo "=== OpenSim Configurator DMG Builder ==="
echo "Version: $VERSION"
echo ""

# --- Preflight ---
if [[ "$(uname)" != "Darwin" ]]; then
    echo "ERROR: This script must be run on macOS." >&2
    exit 1
fi

if ! command -v flutter &>/dev/null; then
    echo "ERROR: flutter not found on PATH." >&2
    exit 1
fi

if [ ! -f "$PROJECT_DIR/pubspec.yaml" ]; then
    echo "ERROR: pubspec.yaml not found. Run from project root or scripts/ dir." >&2
    exit 1
fi

# --- Build ---
echo "[1/4] Building macOS release..."
cd "$PROJECT_DIR"
flutter clean
flutter pub get
flutter build macos --release

APP_PATH="$BUILD_DIR/${APP_NAME}.app"
if [ ! -d "$APP_PATH" ]; then
    echo "ERROR: Build succeeded but ${APP_NAME}.app not found at:" >&2
    echo "  $APP_PATH" >&2
    echo "Checking build output..." >&2
    ls -la "$BUILD_DIR/" 2>/dev/null || echo "  Build directory does not exist"
    exit 1
fi

echo "  Build complete: $(du -sh "$APP_PATH" | cut -f1) app bundle"

# --- Stage ---
echo "[2/4] Staging DMG contents..."
STAGE_CONTENTS="$STAGING_DIR/dmg_contents"
mkdir -p "$STAGE_CONTENTS"
cp -R "$APP_PATH" "$STAGE_CONTENTS/"
ln -s /Applications "$STAGE_CONTENTS/Applications"

# --- Create compressed DMG directly (no intermediate writable image) ---
echo "[3/4] Creating compressed DMG..."
mkdir -p "$DIST_DIR"
FINAL_DMG="$DIST_DIR/${DMG_NAME}.dmg"
rm -f "$FINAL_DMG"

hdiutil create \
    -srcfolder "$STAGE_CONTENTS" \
    -volname "$APP_NAME" \
    -fs HFS+ \
    -fsargs "-c c=64,a=16,e=16" \
    -format UDZO \
    -imagekey zlib-level=9 \
    "$FINAL_DMG"

# --- Summary ---
echo ""
echo "[4/4] Verifying..."
DMG_SIZE=$(du -sh "$FINAL_DMG" | cut -f1)
echo ""
echo "=== Build Complete ==="
echo "  App:     $APP_NAME v$VERSION"
echo "  DMG:     $FINAL_DMG"
echo "  Size:    $DMG_SIZE"
echo "  Bundle:  org.opensimulator.configurator"
echo ""
echo "To install: double-click the .dmg, then drag the app to Applications."
