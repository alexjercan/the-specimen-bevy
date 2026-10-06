{
  description = "Small Bevy game starter";

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
            config.rust-project.crates.horror_game_bevy.crane.outputs.drv.crate;

          assets = builtins.path { path = ./assets; name = "horror_game_bevy-assets"; };
          credits = builtins.path { path = ./credits; name = "horror_game_bevy-credits"; };

          game = pkgs.stdenvNoCC.mkDerivation {
            pname = "horror_game_bevy";
            inherit (unwrapped) version;
            dontUnpack = true;
            nativeBuildInputs = [ pkgs.makeWrapper ];
            installPhase = ''
              mkdir -p $out/bin $out/share/horror_game_bevy
              ln -s ${assets} $out/share/horror_game_bevy/assets
              ln -s ${credits} $out/share/horror_game_bevy/credits
              ln -s ${./LICENSE} $out/share/horror_game_bevy/LICENSE
              makeWrapper ${unwrapped}/bin/horror_game_bevy $out/bin/horror_game_bevy \
                --set-default BEVY_ASSET_ROOT $out/share/horror_game_bevy \
                --prefix LD_LIBRARY_PATH : ${lib.makeLibraryPath gameLibs}
            '';
          };
        in {
          rust-project = {
            toolchain = rustToolchain;
            crates = lib.mkForce {
              horror_game_bevy = {
                path = ./.;
                autoWire = [];
                crane.args = {
                  buildInputs = gameLibs;
                  doCheck = false;
                };
                crane.extraBuildArgs.cargoExtraArgs = "--locked -p horror_game_bevy";
              };
            };
          };

          packages = lib.optionalAttrs pkgs.stdenv.hostPlatform.isLinux {
            default = game;
            horror_game_bevy = game;
            horror_game_bevy-unwrapped = unwrapped;
          };

          apps = lib.optionalAttrs pkgs.stdenv.hostPlatform.isLinux {
            default = { type = "app"; program = "${game}/bin/horror_game_bevy"; };
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
            ];
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
