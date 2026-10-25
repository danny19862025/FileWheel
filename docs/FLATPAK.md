# Flatpak packaging preparation

FileWheel is **not published on Flathub**, and this repository does not yet
contain a buildable Flatpak manifest. These files prepare upstream integration
for a human-maintained package.

## App identity and metadata

The proposed Flatpak ID is `io.github.danny19862025.FileWheel`, matching the
GitHub repository. Its desktop launcher and AppStream metadata are in `data/`.
When `FLATPAK_ID` equals this ID, FileWheel uses it for its GTK application ID,
window icon and tray icon. Native installations keep their existing identity.

After compiling the binary, a packager can install the metadata with:

```sh
sh packaging/install-flatpak-metadata.sh /app
```

The helper installs the launcher, AppStream metadata, existing SVG icon and MIT
license. It does not install the binary or any conversion engines. Run it from
the repository root, inside the package build environment. Use a temporary
prefix instead of `/app` to inspect the output locally.

No release date has been invented in the metadata. Before submission, publish a
tested stable release/tag and add the corresponding AppStream release entry.
The current screenshots are pinned to an existing source commit; update them
if the packaged UI changes.

## Conversion engines

Flatpak cannot discover executables installed by the user's distribution.
The packager must supply these tools inside the sandbox, either through the
selected runtime or reproducible source builds. Inspect what the chosen runtime
actually provides before deciding which additional modules to build.

| Component | FileWheel use |
| --- | --- |
| GTK 4.10 or later | User interface and file chooser |
| gtk4-layer-shell | Linked library for the floating target; required to load the binary |
| ImageMagick 7 (`magick`) | Image conversion and compression; check format delegates and security policy |
| FFmpeg (`ffmpeg`) | Audio, video and subtitle conversion; verify required encoders |
| LibreOffice (`libreoffice`) | Document, presentation and spreadsheet conversions |
| Poppler (`pdftoppm`, `pdftotext`) | PDF image and text export |
| libarchive (`bsdtar`) | Archive operations |
| coreutils (`sha256sum`) | File hashing |
| Python 3 | Standard-library EPUB, text-clipping and gzip adapters |
| qpdf | Optional PDF metadata removal |
| libcanberra with an audio backend | Optional hover sound |

Missing optional tools disable operations in the interface. Do not advertise
all conversions as supported until the packaged engine/codec matrix has passed
tests. LibreOffice is a substantial dependency: installing another app's Flatpak
does not make its executables available here. Do not use unrestricted host
command execution as a shortcut.

Rust dependencies must be available for an offline build from the committed
`Cargo.lock`. Use the maintained Flatpak Cargo tooling and verify checksums.
Select a supported runtime and a compatible Rust SDK extension at packaging
time; neither is pinned by this preparation.

## Sandbox work still required

- File-producing operations save beside the original. File-chooser portal access
  to one input file may not authorize creation of its sibling output. Test this
  explicitly, including files outside the home directory. Decide between a
  documented filesystem grant and an application change to select an output
  location using portals.
- Test drag-and-drop and desktop "Open With" paths, including document-portal
  paths. Do not assume these grant access to neighboring files or directories.
- Test Reveal, Open, Preview and Trash through the sandbox. FileWheel currently
  launches external documents through GIO; investigate any portal adjustments.
- Test Wayland and X11, GNOME and KDE, and layer-shell fallback. Test tray
  registration separately; it calls `org.kde.StatusNotifierWatcher` over D-Bus.
  Grant only the required bus access after checking the actual behavior.
- Verify hover sound and media preview. Conversion runs locally; do not grant
  network access merely to make build-time dependency downloads work.
- Verify appearance settings persist in the Flatpak configuration directory.

## Validation and submission

On a Linux machine with Flatpak and Flathub configured, install the official
builder and validate the metadata:

```sh
flatpak install flathub org.flatpak.Builder
flatpak run --command=flatpak-builder-lint org.flatpak.Builder appstream data/io.github.danny19862025.FileWheel.metainfo.xml
desktop-file-validate data/io.github.danny19862025.FileWheel.desktop
```

Run `cargo test`, `python3 tests/formats_test.py`, and FileWheel's existing
`--self-test` and `--smoke-test` inside the completed package as appropriate.
Build and lint the complete manifest and resulting repository using the official
submission guide before proposing it to Flathub. Metadata preparation alone is
not evidence that a package builds or runs.

### AI disclosure and human ownership

This preparation (metadata, installation helper, identity adaptation and this
document) was generated with AI assistance and needs human review. The maintainer
must also assess and disclose the approximate scope of AI-generated material
elsewhere in the application.

As checked on 2026-10-25, Flathub prohibits AI-generated or AI-assisted manifests.
AI tools must not open or automate Flathub submission PRs, or generate their
commit messages, descriptions or review replies. A human maintainer must write
the manifest and conduct the Flathub submission. Other disclosed AI-generated
material is evaluated at reviewers' discretion. This upstream preparation is
not a Flathub submission.

Consult the live policies before proceeding:

- [Requirements and generative AI policy](https://docs.flathub.org/docs/for-app-authors/requirements/#generative-ai-policy)
- [Submission procedure](https://docs.flathub.org/docs/for-app-authors/submission/)
- [AppStream metadata guidelines](https://docs.flathub.org/docs/for-app-authors/metainfo-guidelines/)
