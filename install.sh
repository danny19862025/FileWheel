#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "$0")"
cargo build --release --locked
install -Dm755 target/release/filewheel "$HOME/.local/bin/filewheel"
install -Dm644 data/local.danny.FileWheel.desktop "$HOME/.local/share/applications/local.danny.FileWheel.desktop"
install -Dm644 data/filewheel.svg "$HOME/.local/share/icons/hicolor/scalable/apps/filewheel.svg"
update-desktop-database "$HOME/.local/share/applications"
printf 'Installed FileWheel. Run ~/.local/bin/filewheel or open it from your application launcher.\n'
