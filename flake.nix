{
  description = "The Specimen, a facility horror game built with Bevy";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    rust-flake.url = "github:juspay/rust-flake";
  };

  outputs = inputs @ { flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [
        inputs.rust-flake.flakeModules.default
        inputs.rust-flake.flakeModules.nixpkgs
      ];

      systems = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin" ];

      perSystem = { config, lib, pkgs, ... }:
        let
          rustToolchain = pkgs.rust-bin.nightly."2026-07-03".default.override {
            extensions = [ "rust-src" "clippy" "rustfmt" ];
            targets = [ "wasm32-unknown-unknown" ];
          };

          gameLibs = lib.optionals pkgs.stdenv.hostPlatform.isLinux (with pkgs; [
            udev
            alsa-lib-with-plugins
            vulkan-loader
            libx11
            libxcursor
            libxi
            libxrandr
            libxkbcommon
            wayland
          ]);

          unwrapped =
            config.rust-project.crates."the-specimen-bevy".crane.outputs.drv.crate;

          assets = builtins.path { path = ./assets; name = "the-specimen-bevy-assets"; };
          credits = builtins.path { path = ./credits; name = "the-specimen-bevy-credits"; };

          game = pkgs.stdenvNoCC.mkDerivation {
            pname = "the-specimen-bevy";
            inherit (unwrapped) version;
            dontUnpack = true;
            nativeBuildInputs = [ pkgs.makeWrapper ];
            installPhase = ''
              mkdir -p $out/bin $out/share/the-specimen-bevy
              ln -s ${assets} $out/share/the-specimen-bevy/assets
              ln -s ${credits} $out/share/the-specimen-bevy/credits
              ln -s ${./LICENSE} $out/share/the-specimen-bevy/LICENSE
              makeWrapper ${unwrapped}/bin/the-specimen-bevy $out/bin/the-specimen-bevy \
                --set-default BEVY_ASSET_ROOT $out/share/the-specimen-bevy \
                --prefix LD_LIBRARY_PATH : ${lib.makeLibraryPath gameLibs}
            '';
          };
        in {
          rust-project = {
            toolchain = rustToolchain;
            crates = lib.mkForce {
              "the-specimen-bevy" = {
                path = ./.;
                autoWire = [];
                crane.args = {
                  buildInputs = gameLibs;
                  doCheck = false;
                };
                crane.extraBuildArgs.cargoExtraArgs = "--locked -p the-specimen-bevy";
              };
            };
          };

          packages = lib.optionalAttrs pkgs.stdenv.hostPlatform.isLinux {
            default = game;
            "the-specimen-bevy" = game;
            "the-specimen-bevy-unwrapped" = unwrapped;
          };

          apps = lib.optionalAttrs pkgs.stdenv.hostPlatform.isLinux {
            default = { type = "app"; program = "${game}/bin/the-specimen-bevy"; };
          };

          checks = lib.optionalAttrs pkgs.stdenv.hostPlatform.isLinux {
            package = game;
            smoke = unwrapped.overrideAttrs (old: {
              src = ./.;
              doNotPostBuildInstallCargoBinaries = true;
              nativeBuildInputs = (old.nativeBuildInputs or []) ++ [ pkgs.python3 pkgs.coreutils ];
              buildPhase = ''
                runHook preBuild
                bash ./tests.sh --smoke
                runHook postBuild
              '';
              installPhase = ''
                runHook preInstall
                mkdir -p $out
                runHook postInstall
              '';
            });
          };

          devShells.default = pkgs.mkShell {
            nativeBuildInputs = with pkgs; [
              rustToolchain rust-analyzer pkg-config python3 cargo-about ffmpeg xvfb-run trunk
            ] ++ lib.optional (lib.meta.availableOn pkgs.stdenv.hostPlatform pkgs.blender) pkgs.blender;
            buildInputs = gameLibs;
            LD_LIBRARY_PATH = lib.makeLibraryPath gameLibs;
            RUST_BACKTRACE = "1";
            BEVY_SOFTWARE_VK_ICD =
              if pkgs.stdenv.hostPlatform.isLinux then
                "${pkgs.mesa}/share/vulkan/icd.d/lvp_icd.${pkgs.stdenv.hostPlatform.parsed.cpu.name}.json"
              else "";
            RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
          };
        };
    };
}
