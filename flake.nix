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
          rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

          # NOTE: share one build lock and one capped sccache server with other Bevy repos.
          cargoWrapper = pkgs.writeShellScriptBin "cargo" ''
            real=${rustToolchain}/bin/cargo
            flock=${pkgs.util-linux}/bin/flock

            # NOTE: nested cargo inherits this marker and must not wait on its parent.
            if [ "''${NOVA_CARGO_LOCKED:-}" = 1 ]; then exec "$real" "$@"; fi

            args=("$@")
            i=0
            sub=
            while [ "$i" -lt "''${#args[@]}" ]; do
              case ''${args[$i]} in
                +*) ;;
                -Z | --config | -C | --color | --explain) i=$((i + 1)) ;;
                -*) ;;
                *) sub=''${args[$i]}; break ;;
              esac
              i=$((i + 1))
            done

            case $sub in
              build | b | check | c | test | t | bench | run | r | clippy | doc | d | rustc | install | fix) ;;
              *) exec "$real" "$@" ;;
            esac

            scope=()
            if command -v systemd-run >/dev/null && [ -S "''${XDG_RUNTIME_DIR:-}/bus" ]; then
              scope=(systemd-run --user --scope --quiet --collect -p MemoryHigh=10G -p MemoryMax=12G --)
            else
              echo "cargo: no systemd user bus, running without the memory cap" >&2
            fi

            take_lock() {
              exec 9>>"''${XDG_RUNTIME_DIR:-/tmp}/cargo-build.lock"
              if ! "$flock" -n 9; then
                echo "cargo: waiting for the build lock (another build is running)" >&2
                "$flock" 9
              fi
              ensure_sccache
            }

            # NOTE: sccache starts rustc outside cargo's scope; cap its server too.
            ensure_sccache() {
              case ''${RUSTC_WRAPPER:-} in */sccache | sccache) ;; *) return ;; esac
              [ "''${#scope[@]}" -gt 0 ] || return
              [ "$(systemctl --user show nova-sccache.scope -p ActiveState --value 2>/dev/null)" = active ] && return
              "$RUSTC_WRAPPER" --stop-server >/dev/null 2>&1
              systemd-run --user --scope --unit=nova-sccache --quiet --collect \
                -p MemoryHigh=10G -p MemoryMax=12G -- "$RUSTC_WRAPPER" --start-server >/dev/null 2>&1 9>&- ||
                echo "cargo: could not start sccache in nova-sccache.scope, compiles run uncapped" >&2
            }

            if [ "$sub" = run ] || [ "$sub" = r ]; then
              # NOTE: build while locked, then release the lock before the game runs.
              build=("''${args[@]:0:i}" build)
              for a in "''${args[@]:i+1}"; do
                [ "$a" = -- ] && break
                build+=("$a")
              done
              take_lock
              NOVA_CARGO_LOCKED=1 "''${scope[@]}" "$real" "''${build[@]}" 9>&- || exit
              exec 9>&-
              exec "''${scope[@]}" "$real" "$@"
            fi

            take_lock
            NOVA_CARGO_LOCKED=1 "''${scope[@]}" "$real" "$@" 9>&-
          '';

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
          };

          devShells.default = pkgs.mkShell {
            nativeBuildInputs = with pkgs; [
              cargoWrapper # NOTE: must precede the toolchain cargo on PATH
              rustToolchain rust-analyzer pkg-config python3 cargo-about ffmpeg xvfb-run trunk
              sccache
              # NOTE: tracy 0.13.1 pairs with the tracy-client-sys pin in Cargo.toml.
              tracy cargo-flamegraph
            ] ++ lib.optional (lib.meta.availableOn pkgs.stdenv.hostPlatform pkgs.blender) pkgs.blender
              ++ lib.optional pkgs.stdenv.hostPlatform.isLinux pkgs.perf;
            buildInputs = gameLibs;
            LD_LIBRARY_PATH = lib.makeLibraryPath gameLibs;
            RUST_BACKTRACE = "1";
            RUSTC_WRAPPER = "sccache";
            CARGO_INCREMENTAL = "0";
            CARGO_BUILD_JOBS = "2";
            BEVY_SOFTWARE_VK_ICD =
              if pkgs.stdenv.hostPlatform.isLinux then
                "${pkgs.mesa}/share/vulkan/icd.d/lvp_icd.${pkgs.stdenv.hostPlatform.parsed.cpu.name}.json"
              else "";
            RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
          };
        };
    };
}
