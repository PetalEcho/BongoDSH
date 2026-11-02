#!/bin/bash
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
    printf '%s\n' 'This script packages a local macOS build only.' >&2
    exit 1
fi

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
source_binary="${1:-$repo_root/target/release/bongodsh}"
app_bundle="$repo_root/target/BongoDSH Preview.app"

if [[ ! -x "$source_binary" ]]; then
    printf '%s\n' 'Build first: cargo build --workspace --release --locked' >&2
    exit 1
fi

mkdir -p "$app_bundle/Contents/MacOS"
install -m 755 "$source_binary" "$app_bundle/Contents/MacOS/bongodsh"
cat > "$app_bundle/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key><string>bongodsh</string>
    <key>CFBundleIdentifier</key><string>dev.bongodsh.preview</string>
    <key>CFBundleName</key><string>BongoDSH Preview</string>
    <key>CFBundleDisplayName</key><string>BongoDSH Preview</string>
    <key>CFBundlePackageType</key><string>APPL</string>
    <key>CFBundleVersion</key><string>1</string>
    <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
PLIST

# Sign the complete bundle so the executable, identifier and Info.plist agree.
# This is a local ad hoc signature, not a distribution identity or notarization.
/usr/bin/codesign --force --sign - "$app_bundle"
/usr/bin/codesign --verify --strict --verbose=2 "$app_bundle"
printf 'Packaged: %s\n' "$app_bundle"
