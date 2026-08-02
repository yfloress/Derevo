{
  description = "Derevo Dev Shell - Rust + Tauri + Svelte (pnpm)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      nixpkgs,
      rust-overlay,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
          config = {
            allowUnfree = true;
            android_sdk.accept_license = true;
          };
        };

        inherit (pkgs) lib;

        # Android toolchain (SDK + NDK) used to cross-compile the native core.
        #
        # Nix evaluates this lazily: defining it downloads nothing. Only the
        # `android` devShell references it, so the default shell never
        # materialises the SDK.
        androidComposition = pkgs.androidenv.composeAndroidPackages {
          # The project Tauri generates targets compileSdk/targetSdk 36 and AGP
          # asks for build-tools 35.0.0. Both must come from the flake because
          # the SDK lives in the read-only Nix store and Gradle cannot install
          # them itself.
          platformVersions = [ "36" ];
          buildToolsVersions = [ "35.0.0" ];
          includeNDK = true;
          ndkVersions = [ "26.3.11579264" ];
        };
        androidSdk = androidComposition.androidsdk;

        androidTargets = [
          "aarch64-linux-android"
          "armv7-linux-androideabi"
          "i686-linux-android"
          "x86_64-linux-android"
        ];

        rustExtensions = [
          "rust-src"
          "rust-analyzer"
          "llvm-tools-preview"
        ];

        # The Android targets add four extra standard libraries, so they are
        # only pulled into the Android shell's toolchain.
        mkRustToolchain =
          targets:
          pkgs.rust-bin.stable.latest.default.override {
            extensions = rustExtensions;
            inherit targets;
          };

        libraries = with pkgs; [
          stdenv.cc.cc.lib
          # Tauri / WebKit
          webkitgtk_4_1
          gtk3
          glib
          cairo
          pango
          gdk-pixbuf
          # System
          openssl
          dbus
          wayland
          libxkbcommon
        ];

        packages = with pkgs; [
          curl
          wget
          pkg-config
          sqlite
          pnpm
          nodejs
          cargo-audit
          cargo-edit
          cargo-modules
          cargo-tauri
          cargo-llvm-cov
          cargo-machete
          cargo-deny
          perl
          python315
          nix-output-monitor
        ];

        androidPackages = with pkgs; [
          cargo-ndk
          android-tools
          jdk17
        ];

        mkDerevoShell =
          {
            withAndroid ? false,
          }:
          pkgs.mkShell {
            buildInputs =
              packages
              ++ libraries
              ++ [ (mkRustToolchain (lib.optionals withAndroid androidTargets)) ]
              ++ lib.optionals withAndroid (androidPackages ++ [ androidSdk ]);

            shellHook =
              ''
                export LIBRARY_PATH=${lib.makeLibraryPath libraries}:$LIBRARY_PATH
                export LD_LIBRARY_PATH=${lib.makeLibraryPath libraries}:$LD_LIBRARY_PATH
                export PKG_CONFIG_PATH=${lib.makeSearchPathOutput "dev" "lib/pkgconfig" libraries}:$PKG_CONFIG_PATH
                export WEBKIT_DISABLE_DMABUF_RENDERER=1

                echo "> DEREVO ${if withAndroid then "ANDROID" else "DEV"} SHELL ACTIVE"
                echo "   Compiler:  Rust $(rustc --version)"
                echo "   Runtime:   pnpm $(pnpm --version)"
              ''
              + lib.optionalString withAndroid ''
                # Android SDK/NDK (cross-compilation through cargo-ndk)
                export ANDROID_HOME="${androidSdk}/libexec/android-sdk"
                export ANDROID_SDK_ROOT="$ANDROID_HOME"
                export ANDROID_NDK_ROOT="$(ls -d "$ANDROID_HOME"/ndk/* 2>/dev/null | head -1)"
                export ANDROID_NDK_HOME="$ANDROID_NDK_ROOT"
                # JDK for the Android Gradle build (tauri android init/build)
                export JAVA_HOME="${pkgs.jdk17.home}"
                # SDK tooling on PATH (emulator, avdmanager, sdkmanager)
                export PATH="$ANDROID_HOME/emulator:$ANDROID_HOME/cmdline-tools/latest/bin:$ANDROID_HOME/platform-tools:$PATH"

                echo "   Android:   NDK $ANDROID_NDK_ROOT"
                echo "   Java:      $(java -version 2>&1 | head -1)"
              '';
          };
      in
      {
        apps.default = {
          type = "app";
          program =
            let
              script = pkgs.writeShellScriptBin "derevo-dev" ''
                exec ${pkgs.cargo-tauri}/bin/cargo-tauri dev "$@"
              '';
            in
            "${script}/bin/derevo-dev";
        };

        devShells = {
          # Daily work and CI: no Android SDK or NDK.
          default = mkDerevoShell { };

          # Only for the Android client: `nix develop .#android`.
          android = mkDerevoShell { withAndroid = true; };
        };
      }
    );
}
