# Verified on 21 October 2026

Environment: Arch Linux, KDE Plasma on Wayland, GTK 4.24.1, gtk4-layer-shell 1.3.0.

- `cargo build --release --locked`: passed without warnings.
- `cargo fmt --check`: passed.
- `cargo test --locked`: 4 tests passed, including output collision protection and killing a running child on cancellation.
- `target/release/filewheel --self-test`: 45 real conversion/tool jobs passed. Checks included decoding image/media results, all-page PDF exports, DOCX text round trips, malformed input cleanup, source preservation and quoted filenames.
- Release GTK smoke: passed mode switching, dispatch through the GTK drop signal, asynchronous conversion, result display, and Wayland layer-shell surface mapping.
- Main window, floating wheel and floating target screenshots were rendered and visually inspected.
- Desktop file validation: passed.
- Installed binary matches the tested release binary: SHA-256 `86afe99f20e78a89b1b2793b4ae15b8e66b75a1ec70af90af6e92a0e42d3445d`.
- Installed to `~/.local/bin/filewheel` and the user application launcher; launched successfully with the floating target enabled.

The smoke test exercises GTK drop routing, not a synthetic physical Dolphin drag. PDF metadata removal was disabled because optional qpdf was absent; it was not tested. Playback previews depend on the GTK media backend installed on the desktop. The app does not globally intercept drags from unrelated Wayland applications: use the floating target.

Renamed to FileWheel: rebuilt release, passed all four unit tests and the GTK smoke test, validated searchable desktop metadata, replaced the installed launcher/command/icon, and preserved old job history. The 45-job conversion suite above was completed before the branding-only rename.

About update: added a reusable native About dialog with the package version (0.1.0) and “Created by Danny”. Release build, formatting check, and the existing GTK conversion smoke test passed; installed binary matched the tested build.

About close/logo fix: embedded the FileWheel PNG in the executable and header, added explicit × and Escape handlers, and passed the GTK regression for × closing, reopening, and closing again. The rendered About dialog was visually checked for its logo, version, and creator credit. Release build and formatting passed; installed binary matched the release.

# Format expansion — 22 October 2026

Added AVIF/BMP image output, the requested audio/video formats, PDF document/text/spreadsheet/presentation outputs, Slides/Spreadsheets/Subtitles/Archives families, document import adapters, all-slide image export, a Formats reference window, per-family notes and desktop MIME registration.

- Release build and `cargo fmt --check`: passed.
- `cargo test --locked`: 7 tests passed, including bundle/file-family routing, import-only menu restrictions and preserving subtitle dialogue that resembles WebVTT headers.
- `python3 tests/formats_test.py`: 3 tests passed, including binary plist text clippings, classic Mac resource forks in AppleDouble sidecars, and EPUB 3 package/reading-order round trips.
- Release `--self-test`: 87 real conversion/tool jobs passed. Newly exercised outputs include AVIF/BMP, all audio/video presets, EPUB import/export, RTFD, text clippings, XLSX/ODS/CSV/TSV/PDF, both subtitle formats and TXT, all-slide exports from a two-slide PPTX, PDF DOCX/PPTX/XLSX/EPUB/TXT/MD, ZIP/TAR/tar.gz repacking, and standalone gzip input. Existing original-preservation and cleanup checks also passed.
- Final release GTK smoke: passed native drop routing, asynchronous conversion/results, About behavior, the nine-choice PDF menu, disabled Pages export and the Formats reference window. Main/PDF/Formats screenshots were rendered and visually inspected.
- Desktop file validation: passed. Updated binary, launcher MIME metadata and icons installed locally.
- Installed/release SHA-256: `918bdd9f2451dfbfa8dc54c481378051c4b7b2cbfbfd159f8e22bb89a4f74b9e`.

Pages export is explicitly unavailable; Pages import and RAR input rely on LibreOffice/libarchive and were not exercised with source fixtures in this expansion. PDF document/spreadsheet exports and EPUB adapters preserve text rather than reconstructing rich layout. PDF-to-PPTX imports page objects through Impress. CSV/TSV export the first worksheet. RTFD attachments are not preserved. See the in-app Formats notes and README for these boundaries.

# Reliability and log privacy — 22 October 2026

- Removed document/clipboard data and converter diagnostics from persisted job summaries. Regression checks the actual written log for both successful and failed jobs.
- Corrected LibreOffice output lookup for names containing multiple dots. The real conversion suite now uses `notes.v2.final.txt`, including DOCX text round trips.
- Isolated converter process groups and kill the full group on cancellation, timeout, or polling failure. Wrapper/descendant regressions cover both cancellation and timeout.
- Moved SHA-256, Copy text, and Word count staging to the system temporary directory. Unit regressions and real LibreOffice/coreutils jobs pass with a readable source in a non-writable directory.
- Added bounded ImageMagick format and FFmpeg encoder discovery. A fixture with absent writers/readers/encoders verifies disabled presets; media encoder checks share the conversion presets.
- `cargo fmt --check`, release build, 13 Rust tests, and 3 Python tests: passed.
- Release `--self-test`: 90 real conversion/tool jobs passed.
- Release GTK smoke: passed drop routing, asynchronous conversion/results, layer-shell mapping, About behavior, PDF menu, and Formats window. GTK emitted the existing slider size warnings.
- Added GitHub Actions for formatting, Rust/Python regressions, and real conversions in an Arch Linux container, running tests without root privileges.

Historical log files are left intact; only newly written entries use the restricted summary. Engine capability lists do not establish that every input decoder, delegate, or runtime policy will allow a conversion. No local installed binary was replaced during this repository update.
