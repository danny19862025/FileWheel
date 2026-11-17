#!/bin/sh
# Install from the repository root after building the application.
# This installs metadata only; it does not create a Flatpak package.
set -eu

if [ "$#" -ne 1 ] || [ -z "$1" ]; then
    echo "Usage: sh packaging/install-flatpak-metadata.sh PREFIX" >&2
    exit 2
fi

prefix=$1
app_id=io.github.danny19862025.FileWheel
install -Dm644 "data/$app_id.desktop" \
    "$prefix/share/applications/$app_id.desktop"
install -Dm644 "data/$app_id.metainfo.xml" \
    "$prefix/share/metainfo/$app_id.metainfo.xml"
install -Dm644 data/filewheel.svg \
    "$prefix/share/icons/hicolor/scalable/apps/$app_id.svg"
install -Dm644 LICENSE "$prefix/share/licenses/$app_id/LICENSE"
