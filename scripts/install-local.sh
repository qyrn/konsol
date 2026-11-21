#!/usr/bin/env bash
set -euo pipefail

project_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
icons_root="$HOME/.local/share/icons/hicolor"
binary_path="$HOME/.local/bin/konsol"

cd "$project_root"
pnpm tauri build --no-bundle

install -Dm755 src-tauri/target/release/konsol "$binary_path"
install -Dm644 src-tauri/icons/32x32.png "$icons_root/32x32/apps/konsol.png"
install -Dm644 src-tauri/icons/64x64.png "$icons_root/64x64/apps/konsol.png"
install -Dm644 src-tauri/icons/128x128.png "$icons_root/128x128/apps/konsol.png"
install -Dm644 src-tauri/icons/128x128@2x.png "$icons_root/256x256/apps/konsol.png"
install -Dm644 src-tauri/icons/icon.png "$icons_root/512x512/apps/konsol.png"

install -Dm644 /dev/stdin "$HOME/.local/share/applications/konsol.desktop" <<DESKTOP
[Desktop Entry]
Type=Application
Name=Konsol
Comment=Lance tous les jeux du PC
Exec=$binary_path
Icon=konsol
StartupWMClass=konsol
Categories=Game;
DESKTOP

kbuildsycoca6 >/dev/null 2>&1 || true
echo "Konsol installé dans $binary_path"
