{
  description = "FileWheel — local file conversion for Linux";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";

  outputs = { self, nixpkgs }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
      pkgsFor = system: import nixpkgs { inherit system; };
      enginesFor = pkgs: with pkgs; [
        imagemagick ffmpeg libreoffice poppler-utils libarchive
        coreutils python3 qpdf xdg-utils curl
      ];
    in {
      packages = forAllSystems (system:
        let pkgs = pkgsFor system;
        in {
          default = self.packages.${system}.filewheel;
          filewheel = pkgs.rustPlatform.buildRustPackage {
            pname = "filewheel";
            version = "0.1.0";
            src = pkgs.lib.cleanSource ./.;
            cargoLock.lockFile = ./Cargo.lock;

            nativeBuildInputs = with pkgs; [ pkg-config wrapGAppsHook4 ];
            buildInputs = with pkgs; [
              gtk4 gtk4-layer-shell fontconfig gsettings-desktop-schemas
              hicolor-icon-theme adwaita-icon-theme librsvg
            ];
            nativeCheckInputs = with pkgs; [ python3 bash coreutils ];

            # Existing subprocess fixtures use FHS paths; use store paths when
            # running those same tests inside the isolated Nix build.
            postPatch = ''
              substituteInPlace src/engine.rs \
                --replace-fail 'Path::new("/bin/sh")' 'Path::new("${pkgs.runtimeShell}")' \
                --replace-fail '#!/bin/sh' '#!${pkgs.runtimeShell}' \
                --replace-fail 'Path::new("/usr/bin/sleep")' 'Path::new("${pkgs.coreutils}/bin/sleep")' \
                --replace-fail 'Path::new("/usr/bin/false")' 'Path::new("${pkgs.coreutils}/bin/false")'
            '';

            doCheck = true;
            postCheck = ''
              python3 tests/formats_test.py
            '';

            postInstall = ''
              install -Dm644 data/local.danny.FileWheel.desktop \
                "$out/share/applications/local.danny.FileWheel.desktop"
              install -Dm644 data/filewheel.svg \
                "$out/share/icons/hicolor/scalable/apps/filewheel.svg"
            '';

            preFixup = ''
              gappsWrapperArgs+=(--prefix PATH : ${pkgs.lib.makeBinPath (enginesFor pkgs)})
              gappsWrapperArgs+=(--prefix LD_LIBRARY_PATH : ${pkgs.lib.makeLibraryPath [ pkgs.libcanberra ]})
            '';

            meta = with pkgs.lib; {
              description = "A local drag-and-drop file converter for Linux";
              homepage = "https://github.com/danny19862025/FileWheel";
              license = licenses.mit;
              mainProgram = "filewheel";
              platforms = systems;
            };
          };
        });

      apps = forAllSystems (system: {
        default = {
          type = "app";
          program = "${self.packages.${system}.filewheel}/bin/filewheel";
          meta.description = "FileWheel desktop file converter";
        };
      });

      checks = forAllSystems (system: {
        filewheel = self.packages.${system}.filewheel;
      });

      devShells = forAllSystems (system:
        let pkgs = pkgsFor system;
        in {
          default = pkgs.mkShell {
            inputsFrom = [ self.packages.${system}.filewheel ];
            packages = with pkgs; [ cargo rustc git desktop-file-utils ] ++ enginesFor pkgs;
            shellHook = ''
              export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath [ pkgs.gtk4 pkgs.gtk4-layer-shell ]}''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
              export XDG_DATA_DIRS="$GSETTINGS_SCHEMAS_PATH:${pkgs.hicolor-icon-theme}/share:${pkgs.adwaita-icon-theme}/share''${XDG_DATA_DIRS:+:$XDG_DATA_DIRS}"
            '';
          };
        });
    };
}
