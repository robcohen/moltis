{
  description = "Moltis - Personal AI gateway inspired by OpenClaw";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
    crane,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        nightly = "2025-11-30";
        wasmCraneLib =
          (crane.mkLib pkgs).overrideToolchain
          (
            p:
              p.rust-bin.nightly.${nightly}.default.override {
                targets = ["wasm32-wasip2"];
              }
          );

        # Pinned nightly to avoid recursion limit overflow in matrix-sdk
        # Latest nightly (2026-04) has query depth changes that break matrix-sdk 0.16
        rustToolchain = pkgs.rust-bin.nightly.${nightly}.default;

        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustToolchain;
          rustc = rustToolchain;
        };

        # Create a clean source that includes necessary files and the wit directory
        src = pkgs.lib.cleanSourceWith {
          src = ./.;
          filter = path: type:
            (pkgs.lib.cleanSourceFilter path type)
            || (builtins.match ".*/wit.*" path != null);
        };

        moltis-wasm-tools = wasmCraneLib.buildPackage {
          inherit src;
          pname = "moltis-wasm-tools";
          doCheck = false;
          cargoExtraArgs = "--target wasm32-wasip2 -p moltis-wasm-calc -p moltis-wasm-web-fetch -p moltis-wasm-web-search ";
          nativeBuildInputs = with pkgs;
            [
              rustPlatform.bindgenHook
              cmake
              perl
              pkg-config
            ]
            ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
              pkgs.libiconv
            ];
        };

        # Web UI assets (Vite + Tailwind + esbuild). The upstream flake omits
        # this step, so the cargo build trips crates/web/build.rs's check for
        # css/style.css, dist/, sw.js. See moltis-org/moltis#441.
        #
        # buildNpmPackage fetches deps reproducibly from package-lock.json
        # (hash pinned via npmDepsHash). The build phase runs
        # `npm run build:all` which writes assets into
        # crates/web/src/assets/{css/style.css, dist/, sw.js}. We capture
        # those and the rust build's preBuild copies them into place.
        moltis-web-assets = pkgs.buildNpmPackage {
          pname = "moltis-web-assets";
          version = "0.1.0";
          src = pkgs.lib.cleanSource ./.;
          sourceRoot = "source/crates/web/ui";

          # Refresh when package-lock.json changes: bump to lib.fakeHash,
          # `nix build`, copy the "got: sha256-..." line from the failure.
          npmDepsHash = "sha256-UxplBZ4a1B31mr5cBWcQzLaB+7psClkXgdZZueI60QM=";

          # build:all = `npm run build && npm run build:css && npm run build:sw`
          # produces dist/, css/style.css (+style.css copy), and sw.js
          # under ../src/assets/ (== crates/web/src/assets/).
          preBuild = ''
            # Vite/Tailwind write output to crates/web/src/assets/; that path
            # may be missing in the unpacked tree (assets are gitignored), so
            # make sure it exists and is writable.
            mkdir -p ../src/assets
            chmod -R u+w ../src
          '';
          buildPhase = ''
            runHook preBuild
            npm run build:all
            runHook postBuild
          '';

          installPhase = ''
            runHook preInstall
            mkdir -p $out
            cp -r ../src/assets/. $out/
            runHook postInstall
          '';

          # We're not publishing an npm package — only consuming built assets.
          dontNpmInstall = true;

          nativeBuildInputs = [ pkgs.nodejs ];
        };
      in {
        packages.default = rustPlatform.buildRustPackage {
          pname = "moltis";
          version = "0.1.0";
          inherit src;
          doCheck = false;

          buildFeatures = [
            "embedded-assets"
            "embedded-wasm"
          ];
          preBuild = ''
            mkdir -p target/wasm32-wasip2/release/
            ln -s ${moltis-wasm-tools}/lib/* target/wasm32-wasip2/release/

            # Drop in the prebuilt web assets so crates/web/build.rs is happy.
            # See moltis-web-assets above.
            mkdir -p crates/web/src/assets
            cp -r ${moltis-web-assets}/. crates/web/src/assets/
            chmod -R u+w crates/web/src/assets
          '';
          cargoLock = {
            lockFile = ./Cargo.lock;
            outputHashes = {
              "sqlx-core-0.8.6" = "sha256-iZZlJ8YGlM1YUEGitK4aZH68tmg3y+gAVysXS8B+DW8=";
            };
          };
          nativeBuildInputs = with pkgs; [
            rustPlatform.bindgenHook
            cmake
            perl
            pkg-config
          ];
          cargoBuildFlags = ["--bin" "moltis"];
          # Match the calendar tag this branch is built on top of. The
          # in-app update checker compares this string against the
          # releases manifest as a calendar version, so a git short-rev
          # here makes the banner permanently misfire. Bump alongside
          # every upstream merge.
          MOLTIS_VERSION = "20260603.01";

          meta = with pkgs.lib; {
            description = "Personal AI gateway inspired by OpenClaw";
            homepage = "https://www.moltis.org/";
            license = licenses.mit;
            mainProgram = "moltis";
          };
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustPlatform.bindgenHook
            pkgs.rust-bin.nightly.${nightly}.default
            rust-analyzer
            cmake
            perl
            pkg-config
          ];
        };
      }
    );
}
