#!/bin/sh
# Wraps the release binary into EncodeTool.app and a .dmg.
# Usage: packaging/make-app.sh [version]   (run ./build.sh first)
set -e
cd "$(dirname "$0")/.."
VERSION="${1:-0.0.0}"
VERSION="${VERSION#v}"
APP="dist/EncodeTool.app"

rm -rf dist && mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp target/release/encodetool "$APP/Contents/MacOS/encodetool"
cp packaging/icon.icns "$APP/Contents/Resources/icon.icns"

cat > "$APP/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>EncodeTool</string>
  <key>CFBundleDisplayName</key><string>EncodeTool</string>
  <key>CFBundleIdentifier</key><string>io.github.ousli.encodetool</string>
  <key>CFBundleExecutable</key><string>encodetool</string>
  <key>CFBundleIconFile</key><string>icon</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>$VERSION</string>
  <key>CFBundleVersion</key><string>$VERSION</string>
  <key>LSMinimumSystemVersion</key><string>12.0</string>
  <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
EOF

# Ad-hoc signature: required for arm64 binaries, but not a Developer ID (Gatekeeper still asks on first launch).
codesign --force --deep -s - "$APP"

STAGE="dist/dmg"
mkdir -p "$STAGE"
cp -R "$APP" "$STAGE/"
ln -s /Applications "$STAGE/Applications"
hdiutil create -quiet -volname EncodeTool -srcfolder "$STAGE" -ov -format UDZO "dist/EncodeTool-$VERSION-macos-arm64.dmg"
rm -rf "$STAGE"
echo "OK : dist/EncodeTool-$VERSION-macos-arm64.dmg"
