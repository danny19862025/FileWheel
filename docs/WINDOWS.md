# FileWheel on Windows

FileWheel runs on Windows 10 (1809 or later) and Windows 11, x86_64. It is the
same app as on Linux: the wheel, the always-on-top floating drop target at the
right edge of the screen, a notification-area (tray) icon, Appearance settings
that follow Windows' light/dark mode, and the same conversions.

## Install with the installer (recommended)

1. Download `FileWheel-Setup-x86_64.exe` from the
   [latest release](https://github.com/danny19862025/FileWheel/releases/latest).
2. Run it. Windows SmartScreen may warn because the build is not code-signed:
   choose **More info → Run anyway**.
3. Follow the wizard. It installs for your account only (no administrator
   prompt) into `%LOCALAPPDATA%\Programs\FileWheel`, adds a Start menu entry,
   and can optionally add a desktop shortcut and start the drop target at
   sign-in.

Uninstall from **Settings → Apps → Installed apps**. Settings in
`%APPDATA%\FileWheel` are kept; delete it and `%LOCALAPPDATA%\FileWheel` to
remove them as well.

## Or use the portable build

1. Download `FileWheel-windows-x86_64.zip` from the
   [latest release](https://github.com/danny19862025/FileWheel/releases/latest),
   or from the latest successful **CI** run's artifacts.
2. Extract it anywhere, for example `C:\Users\<you>\Apps\FileWheel`.
3. Run `FileWheel.exe`.

Nothing is installed or registered. Settings are stored in
`%APPDATA%\FileWheel`, and the job log and LibreOffice profile in
`%LOCALAPPDATA%\FileWheel`. To remove FileWheel, quit it and delete the folder
and those two folders.

## Install the conversion engines

FileWheel uses free conversion engines that you install once. Each is
optional: without one, only the operations that need it are disabled, and
**Conversion engines** in FileWheel lists what is missing with the matching
command. From PowerShell or Command Prompt:

```powershell
winget install -e --id ImageMagick.ImageMagick
winget install -e --id Gyan.FFmpeg
winget install -e --id TheDocumentFoundation.LibreOffice
winget install -e --id oschwartz10612.Poppler
winget install -e --id Python.Python.3.13
winget install -e --id QPDF.QPDF
```

Restart FileWheel afterwards. FileWheel looks on `PATH` and in each engine's
default install folder, so a new sign-in is not required.

| Engine | Used for |
| --- | --- |
| ImageMagick | Image conversion and image tools |
| FFmpeg | Audio, video, GIF and subtitle conversion |
| LibreOffice | Documents, slides, spreadsheets, PDF → DOCX/PPTX/XLSX |
| Poppler | PDF → JPG/PNG/TXT/MD, slides → images |
| Python 3 | EPUB, text clippings and gzip input |
| QPDF | PDF metadata removal |

ZIP, TAR and GZIP use the `tar.exe` (libarchive) built into Windows 10 and 11.
SHA-256 needs no engine.

## Differences from Linux

- **Drop target:** an always-on-top window at the right edge of the screen
  instead of a Wayland layer-shell surface. Drag files from File Explorer onto
  it, then onto a wheel segment.
- **Tray icon:** click to open FileWheel; right-click for *Show/hide drop
  target* and *Quit*. Closing the main window keeps FileWheel running in the
  tray.
- **Reveal** opens File Explorer with the first file selected.
- **Preview** shows images and text inside FileWheel; audio and video open in
  your default player.
- The hover sound plays through Windows' own audio; no extra library needed.

## Build from source

FileWheel is built with [MSYS2](https://www.msys2.org/)'s UCRT64 environment.
In an **MSYS2 UCRT64** shell:

```bash
pacman -S --needed git zip mingw-w64-ucrt-x86_64-{gtk4,rust,pkgconf,gcc}
git clone https://github.com/danny19862025/FileWheel.git
cd FileWheel
cargo build --release --locked
bash packaging/windows/package.sh
```

The portable folder is `dist/FileWheel`, and the ZIP is
`dist/FileWheel-windows-x86_64.zip`. To build the installer as well, install
[Inno Setup 6](https://jrsoftware.org/isdl.php) and run
`ISCC.exe /DAppVersion=0.1.0 packaging\windows\FileWheel.iss`, which writes
`dist\FileWheel-Setup-x86_64.exe`. `cargo test --locked` runs the test
suite; tests that need ImageMagick or FFmpeg are skipped when those are
missing.
