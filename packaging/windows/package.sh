#!/usr/bin/env bash
# Assemble the portable Windows build from an MSYS2 UCRT64 shell:
#   cargo build --release --locked && bash packaging/windows/package.sh
# Produces dist/FileWheel/ and dist/FileWheel-windows-x86_64.zip.
#
# Layout: FileWheel.exe sits next to its DLLs, with share/ and lib/ beside
# it. GLib locates its data relative to the executable's folder.
set -euo pipefail
cd -- "$(dirname -- "$0")/../.."

prefix=${MINGW_PREFIX:?Run this from an MSYS2 UCRT64 shell}
exe=${CARGO_TARGET_DIR:-target}/release/filewheel.exe
out=dist/FileWheel
rm -rf dist
mkdir -p "$out"

cp "$exe" "$out/FileWheel.exe"
# GApplication uses D-Bus for single-instance handling; on Windows GLib
# starts this helper on demand.
cp "$prefix/bin/gdbus.exe" "$out/"

pixbuf=lib/gdk-pixbuf-2.0/2.10.0
mkdir -p "$out/$pixbuf/loaders"
cp "$prefix/$pixbuf/loaders.cache" "$out/$pixbuf/"
cp "$prefix/$pixbuf"/loaders/*.dll "$out/$pixbuf/loaders/"

# Copy every DLL from the MSYS2 prefix that the programs and loaders need.
declare -A seen=()
queue=("$out/FileWheel.exe" "$out/gdbus.exe" "$out/$pixbuf"/loaders/*.dll)
while ((${#queue[@]})); do
    file=${queue[0]}
    queue=("${queue[@]:1}")
    while read -r dll; do
        name=$(basename "$dll")
        [[ -n ${seen[$name]:-} ]] && continue
        seen[$name]=1
        cp "$dll" "$out/"
        queue+=("$out/$name")
    done < <(ldd "$file" | awk -v p="$prefix/bin/" 'index($3, p) == 1 { print $3 }')
done

mkdir -p "$out/share/glib-2.0/schemas" "$out/share/icons/hicolor"
cp "$prefix/share/glib-2.0/schemas/gschemas.compiled" "$out/share/glib-2.0/schemas/"
cp "$prefix/share/icons/hicolor/index.theme" "$out/share/icons/hicolor/"

# License texts for the bundled libraries (GTK, GLib and friends are LGPL).
mkdir -p "$out/licenses"
cp LICENSE "$out/licenses/FileWheel.txt"
cp data/fonts/Ribeye-OFL.txt "$out/licenses/"
for name in "${!seen[@]}"; do
    package=$(pacman -Qqo "$prefix/bin/$name" 2>/dev/null) || continue
    package=${package#"${MINGW_PACKAGE_PREFIX}-"}
    if [[ -d $prefix/share/licenses/$package && ! -e $out/licenses/$package ]]; then
        cp -r "$prefix/share/licenses/$package" "$out/licenses/"
    fi
done

cat > "$out/README.txt" <<'EOF'
FileWheel for Windows (portable)
================================

Run FileWheel.exe. Nothing is installed; delete this folder to remove it.
Settings are kept in %APPDATA%\FileWheel.

FileWheel converts files with free engines you install separately. Open
"Conversion engines" in FileWheel to see which are missing, or install them
all from PowerShell or Command Prompt, then restart FileWheel:

  winget install -e --id ImageMagick.ImageMagick
  winget install -e --id Gyan.FFmpeg
  winget install -e --id TheDocumentFoundation.LibreOffice
  winget install -e --id oschwartz10612.Poppler
  winget install -e --id Python.Python.3.13
  winget install -e --id QPDF.QPDF

Each engine is optional: without it, only the operations that need it are
disabled. ZIP/TAR archives use the tar.exe built into Windows 10 and 11.

Windows SmartScreen may warn the first time because this build is not
code-signed. Choose "More info", then "Run anyway".

Project page: https://github.com/danny19862025/FileWheel
EOF

(cd dist && zip -qr9 FileWheel-windows-x86_64.zip FileWheel)
du -sh "$out" dist/FileWheel-windows-x86_64.zip
