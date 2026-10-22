# Linux installation guide

FileWheel builds from source. Choose your distribution below, install its
dependencies, then follow [Build and install](#build-and-install). NixOS users
should use the [flake guide](NIXOS.md), which also covers Home Manager.

Package names were checked against upstream distribution sources on 22 October
2026. The application has been exercised on Arch Linux/KDE Wayland. The other
sections are installation recipes, not claims that every distribution, release,
architecture, or desktop has been tested.

## Choose your distribution

| Distribution or family | Instructions |
| --- | --- |
| Arch, EndeavourOS, Garuda, CachyOS, Manjaro | [Arch family](#arch-family) |
| Ubuntu, Kubuntu, Xubuntu, Mint, Pop!_OS, elementary OS, Zorin | [Ubuntu family](#ubuntu-family) |
| Debian and Debian-based distributions | [Debian](#debian) |
| Fedora Workstation, KDE and other mutable Fedora spins | [Fedora](#fedora) |
| NixOS, or another Linux distribution with Nix | [Nix flakes, NixOS and Home Manager](NIXOS.md) |
| Gentoo | [Gentoo](#gentoo) |
| openSUSE Tumbleweed, Slowroll, Leap | [openSUSE](#opensuse) |
| Void Linux | [Void](#void) |
| Alpine Linux | [Alpine](#alpine) |
| Fedora Atomic, Universal Blue, Bazzite, SteamOS, Aeon | [Immutable desktops](#immutable-desktops) |
| RHEL, Rocky, AlmaLinux, CentOS Stream, Oracle Linux | [Enterprise distributions](#enterprise-distributions) |
| Solus, Mageia, OpenMandriva, Slackware, other Linux systems | [Other distributions](#other-distributions) |

Derivatives inherit the requirements of their **base release**. For example,
an Ubuntu 24.04-based desktop does not gain newer packages by using the Ubuntu
26.04 instructions.

## Requirements

| Component | Requirement or purpose |
| --- | --- |
| Rust and Cargo | Current stable Rust with edition 2024 support, compatible with `Cargo.lock` |
| C compiler, linker, `pkg-config` | Build the Rust/GTK application |
| GTK4 development files | GTK **4.10 or newer** |
| gtk4-layer-shell development files | Required at link time, including on X11/GNOME |
| Git, Bash, GNU coreutils, desktop-file-utils | Fetch source, run `install.sh`, register the launcher |
| ImageMagick **7** (`magick`) | Image conversions and image tools |
| FFmpeg (`ffmpeg`, `ffprobe`) | Audio/video/subtitle conversions; encoders vary by distribution |
| LibreOffice | Office document, presentation and spreadsheet conversions |
| Poppler tools (`pdftoppm`, `pdftotext`) | PDF image/text exports |
| libarchive tools (`bsdtar`) | Archive conversion and ZIP tools |
| Python 3 (`python3`) | EPUB, macOS text clipping and gzip adapters |
| `sha256sum` | Hashing; normally provided by GNU coreutils |
| qpdf | Optional PDF metadata removal |
| xdg-utils, a desktop session and fonts | Desktop integration and useful document rendering |

Conversion engines are optional for launching FileWheel: missing engines disable
their operations. GTK4 and gtk4-layer-shell are build/runtime library requirements.
The GTK3 library named `gtk-layer-shell` is **not** a replacement for
`gtk4-layer-shell`.

## Arch family

On Arch and derivatives with equivalent repository packages:

```sh
sudo pacman -Syu --needed base-devel git rust gtk4 gtk4-layer-shell \
  desktop-file-utils xdg-utils imagemagick ffmpeg libreoffice-fresh \
  poppler libarchive python coreutils qpdf ttf-dejavu
```

`libreoffice-still` can replace `libreoffice-fresh`; choose one. Manjaro and other
derivatives can ship different versions, so check the prerequisites below if a
package is unavailable. Do not mix Arch and Manjaro repositories.

Sources: [Arch gtk4-layer-shell](https://archlinux.org/packages/extra/x86_64/gtk4-layer-shell/),
[Arch Rust](https://archlinux.org/packages/extra/x86_64/rust/).

## Ubuntu family

Use this recipe for Ubuntu **26.04 LTS** and derivatives based on that release.
Enable Ubuntu's Universe repository if necessary through Software & Updates or
`sudo add-apt-repository universe`.

```sh
sudo apt update
sudo apt install build-essential git cargo rustc pkg-config \
  libgtk-4-dev libgtk4-layer-shell-dev desktop-file-utils xdg-utils \
  imagemagick ffmpeg libreoffice poppler-utils libarchive-tools \
  python3 coreutils qpdf fonts-dejavu-core
```

Then check `rustc --version` and `magick -version` before building. If the
distribution's Rust is too old for the locked dependencies, install current
stable Rust using the [official rustup instructions](https://rust-lang.org/tools/install/)
and open a new terminal.

Ubuntu **24.04 LTS** has a sufficiently new GTK4, but its standard repositories
do not provide the same GTK4 layer-shell/ImageMagick 7 combination as this recipe.
Ubuntu **22.04 LTS** also has a GTK4 version below FileWheel's minimum. Mint,
Pop!_OS, elementary OS and Zorin releases based on these versions inherit those
constraints. Use the [Nix flake](NIXOS.md), a newer base release, or the
[missing-library instructions](#missing-gtk4-layer-shell). Renaming `convert`
to `magick` does not make ImageMagick 6 compatible.

Sources: [Ubuntu GTK4 layer-shell development package](https://packages.ubuntu.com/resolute/libgtk4-layer-shell-dev),
[Ubuntu ImageMagick 7](https://packages.ubuntu.com/resolute/imagemagick).

## Debian

For Debian **13 (trixie)** or a newer suite containing these packages:

```sh
sudo apt update
sudo apt install build-essential git cargo rustc pkg-config \
  libgtk-4-dev libgtk4-layer-shell-dev desktop-file-utils xdg-utils \
  imagemagick ffmpeg libreoffice poppler-utils libarchive-tools \
  python3 coreutils qpdf fonts-dejavu-core
```

Use current stable Rust through rustup if packaged Rust cannot build the locked
dependencies. Debian 12's default GTK4 is too old, and its ImageMagick is version
6. Use a newer distribution or the Nix route rather than mixing stable/testing
libraries into a working system.

Sources: [Debian trixie layer-shell packages](https://packages.debian.org/source/trixie/gtk4-layer-shell),
[Debian ImageMagick versions](https://sources.debian.org/src/imagemagick/).

## Fedora

On a current mutable Fedora release:

```sh
sudo dnf install gcc git rust cargo pkgconf-pkg-config \
  gtk4-devel gtk4-layer-shell-devel desktop-file-utils xdg-utils \
  ImageMagick ffmpeg-free libreoffice-writer libreoffice-calc \
  libreoffice-impress poppler-utils bsdtar python3 coreutils qpdf \
  dejavu-sans-fonts
```

Fedora's `ffmpeg-free` exposes fewer codecs than some other FFmpeg builds.
FileWheel disables explicit presets with missing encoders. If you need additional
codecs, follow [Fedora's multimedia guidance](https://docs.fedoraproject.org/en-US/quick-docs/installing-plugins-for-playing-movies-and-music/)
for your release and chosen repositories. Restart FileWheel afterward.

Sources: [Fedora GTK4 layer-shell development package](https://packages.fedoraproject.org/pkgs/gtk4-layer-shell/gtk4-layer-shell-devel/),
[FFmpeg packages](https://packages.fedoraproject.org/pkgs/ffmpeg/),
[bsdtar package](https://packages.fedoraproject.org/pkgs/libarchive/bsdtar/).

## Gentoo

Install the build libraries and conversion tools from your configured Portage tree:

```sh
sudo emerge --ask dev-vcs/git dev-lang/rust virtual/pkgconfig \
  gui-libs/gtk:4 gui-libs/gtk4-layer-shell dev-util/desktop-file-utils \
  x11-misc/xdg-utils media-gfx/imagemagick media-video/ffmpeg \
  app-office/libreoffice app-text/poppler app-arch/libarchive \
  dev-lang/python sys-apps/coreutils app-text/qpdf
```

You need a working C toolchain, GTK4 ≥ 4.10 and ImageMagick 7. `dev-lang/rust-bin`
can supply the Rust toolchain instead of building Rust from source. Select the
LibreOffice source or binary package that fits your system.

Review USE flags with `emerge --pretend --verbose` before merging. Enable GTK4's
`wayland` flag for a Wayland session, Poppler's `utils` flag for `pdftoppm` and
`pdftotext`, and the desired FFmpeg encoders/ImageMagick delegates. Office PDF
import may require LibreOffice's `pdfimport` flag. Resolve keyword or USE changes
for your profile rather than globally enabling unstable packages.

Sources: [Gentoo GTK4 layer-shell](https://packages.gentoo.org/packages/gui-libs/gtk4-layer-shell),
[GTK4 ebuild](https://github.com/gentoo/gentoo/tree/master/gui-libs/gtk),
[Poppler flags](https://packages.gentoo.org/packages/app-text/poppler),
[LibreOffice flags](https://packages.gentoo.org/packages/app-office/libreoffice).

## openSUSE

On Tumbleweed/Slowroll, first check which packages your configured repositories
provide:

```sh
zypper search -s gtk4-layer-shell
zypper search --provides bsdtar
```

Install the common dependencies:

```sh
sudo zypper install gcc gcc-c++ make git rust cargo pkg-config \
  gtk4-devel desktop-file-utils xdg-utils ImageMagick ffmpeg \
  libreoffice-writer libreoffice-calc libreoffice-impress \
  poppler-tools python3 coreutils qpdf
```

If available in your selected repositories, also install the GTK4 layer-shell
development package (commonly `gtk4-layer-shell-devel`) and the package providing
`bsdtar` returned by the search above.
If GTK4 layer-shell is absent, use the [upstream build route](#missing-gtk4-layer-shell)
or Nix. The availability of community OBS builds does not establish that a
package is included in a release's default repositories.

Leap users must check GTK ≥ 4.10 and ImageMagick 7 first; do not use Tumbleweed
repositories to replace Leap's core libraries. Codec availability depends on the
enabled repositories and FFmpeg build.

Sources: [openSUSE package search](https://software.opensuse.org/),
[upstream GTK4 layer-shell installation](https://github.com/wmww/gtk4-layer-shell#installation).

## Void

```sh
sudo xbps-install -S
sudo xbps-install base-devel git rust cargo pkg-config gtk4-devel \
  gtk4-layer-shell-devel desktop-file-utils xdg-utils ImageMagick \
  ffmpeg libreoffice poppler bsdtar python3 coreutils qpdf
```

Build on the same libc variant you run: a glibc build is not a portable musl
build. Some engines may not be packaged for every Void architecture/libc target;
missing engines leave their operations disabled.

Sources: [Void GTK4 layer-shell template](https://github.com/void-linux/void-packages/blob/master/srcpkgs/gtk4-layer-shell/template),
[ImageMagick template](https://github.com/void-linux/void-packages/blob/master/srcpkgs/ImageMagick/template),
[libarchive/bsdtar template](https://github.com/void-linux/void-packages/blob/master/srcpkgs/libarchive/template).

## Alpine

Use a current Alpine release with its matching `community` repository enabled:

```sh
sudo apk add build-base bash git rust cargo pkgconf gtk4.0-dev \
  gtk4-layer-shell-dev desktop-file-utils xdg-utils imagemagick \
  ffmpeg libreoffice poppler-utils libarchive-tools python3 \
  coreutils qpdf font-dejavu
```

Use `doas` instead of `sudo` if that is your configured privilege tool. Check
`magick -version`, `rustc --version`, and GTK's version before building. Do not
mix Alpine edge packages into a stable release to resolve missing dependencies.
Build locally for musl; the Arch-built binary is not an Alpine binary.

Sources: [Alpine GTK4 development files](https://pkgs.alpinelinux.org/packages?name=gtk4.0-dev),
[GTK4 layer-shell](https://pkgs.alpinelinux.org/packages?name=gtk4-layer-shell-dev),
[libarchive tools](https://pkgs.alpinelinux.org/packages?name=libarchive-tools).

## Immutable desktops

For Fedora Atomic, Kinoite, Silverblue, Universal Blue/Bazzite, SteamOS and Aeon,
prefer the Nix route where supported, or a desktop-enabled Distrobox/Toolbox
container. Do not assume a binary built in a container can run directly on the
host with its different libraries.

For example, with Distrobox already installed and configured:

```sh
distrobox create --name filewheel-dev --image docker.io/library/archlinux:latest
distrobox enter filewheel-dev
```

Inside the container, follow the Arch dependencies and source build instructions.
Start FileWheel **inside** that container. Optional launcher export, run inside
the container after installation:

```sh
distrobox-export --app "$HOME/.local/share/applications/local.danny.FileWheel.desktop"
```

Distrobox commonly shares your home directory. Converted files are real files
there, not isolated test copies. Wayland, the tray, Trash, media previews and
floating targets depend on desktop integration passed through by the container;
start by testing the main-window file drop.

Source: [Distrobox application export](https://distrobox.it/usage/distrobox-export/).

## Enterprise distributions

For RHEL, Rocky, AlmaLinux, CentOS Stream and Oracle Linux, first compare the
release's GTK4, Rust, ImageMagick and layer-shell packages with the requirements.
These distributions do not necessarily have Fedora's packages or codec selection.
Use the Nix flake on supported Linux systems or a current desktop container when
the host repositories are missing those dependencies. The Fedora command above
is not an enterprise-Linux installation command.

## Other distributions

For Solus, Mageia, OpenMandriva, Slackware and other systems, use your package
manager's search/provider tools to locate the components in the requirements
table. Package names vary; the executable and library checks below are the
common contract. Once the prerequisites are available, the same source build
works without a distribution-specific installer. If they are unavailable, use
Nix where supported or a desktop container with a supported dependency set.

## Check the prerequisites

```sh
rustc --version
cargo --version
pkg-config --atleast-version=4.10 gtk4
pkg-config --modversion gtk4
pkg-config --modversion gtk4-layer-shell-0
magick -version
ffmpeg -hide_banner -encoders
```

The `--atleast-version` command returns success only when GTK is new enough.
To see which optional tools are visible:

```sh
for tool in magick ffmpeg ffprobe libreoffice pdftoppm pdftotext \
  bsdtar python3 sha256sum qpdf update-desktop-database; do
  command -v "$tool" || printf 'Missing: %s\n' "$tool"
done
```

## Build and install

Run as your normal desktop user after installing dependencies:

```sh
git clone https://github.com/danny19862025/FileWheel.git
cd FileWheel
cargo build --release --locked
bash install.sh
export PATH="$HOME/.local/bin:$PATH"
filewheel
```

`install.sh` builds the release if needed and installs only into your user
account. Do **not** run it with `sudo`. Add `~/.local/bin` to your shell/session
PATH permanently if it is not already there. Nix users should install the
wrapped flake package instead of using this script.

Use **Conversion engines** in FileWheel to inspect installed tools. Disabled
formats show the missing engine/codec. Reopen the app after installing codecs.

For an existing clone, quit FileWheel, run `git pull --ff-only`, and repeat
`bash install.sh`. Review any local changes before updating.

## Missing GTK4 layer-shell

Even on a desktop without the Wayland layer-shell protocol, the current binary
still links against the GTK4 layer-shell library. A missing runtime library
prevents startup; a missing development library prevents building.

If your distribution has GTK ≥ 4.10 but no suitable GTK4 layer-shell package,
follow the [upstream source-build instructions](https://github.com/wmww/gtk4-layer-shell#building-from-source).
They use Meson/Ninja plus GTK4 and Wayland development files. Choose a released
version and follow your distribution's local-library registration procedure.
Then repeat the `pkg-config` checks above. The Nix flake supplies the library
without changing system library paths.

## Desktop and conversion troubleshooting

| Symptom | What to check |
| --- | --- |
| GTK version/build error | Install GTK4 development files ≥ 4.10, not GTK3; check `pkg-config` |
| Cannot link/find `gtk4-layer-shell` | Install its GTK4 development/runtime package; GTK3 layer-shell is different |
| `filewheel` or launcher not found | Confirm `~/.local/bin` is in the desktop session's PATH, or launch `~/.local/bin/filewheel` |
| No image conversions | Check that `magick` is ImageMagick 7, not just ImageMagick 6's `convert` |
| Some HEIC/AVIF/media outputs disabled | Install matching delegates/encoders; inspect `magick -list format` and `ffmpeg -encoders` |
| PDF image export fails despite ImageMagick | Inspect its PDF security policy and delegates; follow distro policy documentation rather than disabling all restrictions |
| Office conversion unavailable | Install Writer/Calc/Impress and their filters; older LibreOffice builds may lack Markdown/Pages features |
| Floating target unavailable | KDE/wlroots Wayland generally supports layer-shell; GNOME's compositor does not provide that protocol; use the main window or regular-window fallback |
| No tray icon | Check desktop StatusNotifier/AppIndicator support before using **Hide** |
| Video/audio preview fails | Install the GTK media backend and matching GStreamer plugins; preview support is separate from FFmpeg conversion |
| Source directory is read-only | Hash/Copy text/Word count work without writing there; file outputs still need a writable destination beside the source |

Run on a graphical Linux desktop. Windows/WSL without a suitable graphical
session, macOS, and FreeBSD are not covered by these Linux recipes.

## Verify a build

```sh
cargo fmt --check
cargo test --locked
python3 tests/formats_test.py
target/release/filewheel --self-test
```

Run Rust tests without root privileges. The full conversion self-test expects
the fixture tools, codecs and filters it exercises; an intentionally minimal
installation may not pass every conversion. On Nix, use the flake's packaged
checks, which adapt subprocess fixture paths to the Nix store.

In a graphical session, you can also run
`target/release/filewheel --smoke-test`. This opens test windows briefly and
checks GTK drop routing and result display.
