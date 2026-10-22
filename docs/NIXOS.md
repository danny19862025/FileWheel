# Nix flakes, NixOS and Home Manager

FileWheel includes a [flake](../flake.nix) for `x86_64-linux` and `aarch64-linux`.
It builds the Rust application, installs its launcher/icon, and wraps it with
GTK settings and conversion engines on PATH. You do not need to add all the
conversion engines separately to your system configuration.

The wrapper also supplies optional `libcanberra` for soft hover sounds. Choose
your theme, custom accent and sound/glow switches from **Appearance**. Your host
desktop still provides the audio session and the portal used by Follow system.

The flake uses Nixpkgs 26.05 with the revision pinned in `flake.lock`. ARM output
is provided as a recipe; it has not been tested on ARM hardware. Graphical
behavior still depends on the desktop/compositor described in the
[Linux guide](INSTALL.md#desktop-and-conversion-troubleshooting).

## Enable flakes

On NixOS, merge this setting into your existing configuration and rebuild it:

```nix
nix.settings.experimental-features = [ "nix-command" "flakes" ];
```

On other Linux systems with Nix, add this line to `~/.config/nix/nix.conf`:

```ini
experimental-features = nix-command flakes
```

For one command without changing configuration, pass
`--extra-experimental-features 'nix-command flakes'` to `nix`.
See [Nix's flake documentation](https://nix.dev/concepts/flakes).

## Run without adding it to your profile

```sh
nix run github:danny19862025/FileWheel
```

Pass files or app options after `--`:

```sh
nix run github:danny19862025/FileWheel -- "$HOME/Pictures/photo.png"
nix run github:danny19862025/FileWheel -- --background
```

The first build may download substantial GTK, office and media dependencies.
Run it from your graphical desktop session as your normal user.

## Install for your user

```sh
nix profile add github:danny19862025/FileWheel#filewheel
filewheel
```

Older Nix versions may spell the first command `nix profile install` instead of
`nix profile add`. The profile includes the desktop entry and icon. Log out and
back in if your desktop session does not yet see the updated profile's
applications directory.

## Add it to a NixOS system flake

Merge the `filewheel` input and package into your existing `flake.nix`. Keep your
own hostname, architecture, hardware configuration and modules. This is the
relevant structure, not a replacement for your whole configuration:

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
    filewheel.url = "github:danny19862025/FileWheel";
  };

  outputs = { nixpkgs, filewheel, ... }: {
    nixosConfigurations.my-host = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux"; # use aarch64-linux on an ARM Linux machine
      modules = [
        ./configuration.nix
        ({ pkgs, ... }: {
          environment.systemPackages = [
            filewheel.packages.${pkgs.stdenv.hostPlatform.system}.filewheel
          ];
        })
      ];
    };
  };
}
```

From your system configuration directory, replace `my-host` with your existing
configuration name:

```sh
sudo nixos-rebuild switch --flake .#my-host
```

FileWheel keeps its own pinned Nixpkgs input by default. You may add
`filewheel.inputs.nixpkgs.follows = "nixpkgs";` to share your system's Nixpkgs,
but that uses your chosen GTK/Rust/codec versions instead of FileWheel's pin.

## Add it through Home Manager

If Home Manager is integrated into your NixOS flake, pass the input to its modules:

```nix
home-manager.extraSpecialArgs = { inherit filewheel; };
```

For standalone Home Manager, add `filewheel` to your existing flake's inputs and
set `extraSpecialArgs = { inherit filewheel; };` in the existing
`home-manager.lib.homeManagerConfiguration` call.

Then merge this into your `home.nix` (including the function argument):

```nix
{ pkgs, filewheel, ... }:
{
  home.packages = [
    filewheel.packages.${pkgs.stdenv.hostPlatform.system}.filewheel
  ];
}
```

Apply the appropriate existing configuration with `home-manager switch --flake
.#your-profile`, or your normal `nixos-rebuild` command when integrated into
NixOS. See the [Home Manager manual](https://nix-community.github.io/home-manager/)
for its system/standalone flake setup.

## Build, check and develop from a clone

```sh
git clone https://github.com/danny19862025/FileWheel.git
cd FileWheel
nix build .#filewheel
./result/bin/filewheel
nix flake check
```

The package enables Rust unit tests and Python adapter tests during its build.
It adjusts the existing subprocess test fixtures' `/bin/sh` and `/usr/bin/*`
paths to store paths inside the Nix build. `nix flake check` checks the native
system; the flake advertises ARM outputs without cross-building them here.

For source development:

```sh
nix develop
cargo build --release --locked
target/release/filewheel
```

Run the directly built binary inside that development shell. For an installed
app, use `nix build`, `nix run`, your Nix profile or the declarative package so
the GTK/engine wrapper is retained. Do not copy the raw Cargo binary into a
profile or run `install.sh` as a substitute for Nix packaging.

The full conversion suite can be run from `nix develop` with
`target/release/filewheel --self-test`; desktop smoke tests also need a graphical
session. The regular host `cargo test` fixtures still assume FHS paths; use
`nix flake check` for the adapted tests on NixOS.

## Update the locked inputs

For a FileWheel source clone:

```sh
nix flake update nixpkgs
nix flake check
```

Review and commit the changed `flake.lock` when maintaining the project. For a
system/Home Manager flake, update **its** `filewheel` input and rebuild instead:

```sh
nix flake update filewheel
```

If editing a Git checkout, add new Nix/source files with `git add` before Nix
evaluates them; flakes based on Git omit untracked files.

The GTK wrapper follows the [Nixpkgs GTK packaging hooks](https://nixos.org/manual/nixpkgs/unstable/#sec-language-gnome).
The package also supplies [`gtk4-layer-shell`](https://github.com/NixOS/nixpkgs/blob/nixos-26.05/pkgs/by-name/gt/gtk4-layer-shell/package.nix)
and Poppler's `poppler-utils` output. It does not enable autostart, change your
compositor or set default file associations.
