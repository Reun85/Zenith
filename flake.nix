{
  description = "Test Rust project";

  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { flake-parts, ... }@inputs:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" "x86_64-darwin" ];
      perSystem = { config, lib, system, ... }:
        let
          # overlay Cargo with nightly?
          pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [ inputs.rust-overlay.overlays.default ];
          };
          # load in used rust Toolchain to ensure that toolchain is downloaded
          rustTools = (pkgs.rust-bin.fromRustupToolchainFile
            ./rust-toolchain.toml).override {
              extensions = [ "rust-analyzer" "rust-src" "clippy" "rustfmt" ];
            };
        in {
          devShells.default = pkgs.mkShell {
            pureShell = true;
            # build time dependencies
            nativeBuildInputs = with pkgs;
              [ pkg-config mold clang cmake python3 cargo-watch rustTools ] ++
              # vulkan
              [
                glslang
                shaderc # GLSL to SPIRV compiler - glslc
                shaderc.bin
                shaderc.static
                shaderc.dev
                shaderc.lib
                vulkan-headers
                vulkan-loader
                vulkan-validation-layers
                vulkan-tools # vulkaninfo
                renderdoc # Graphics debugger
                tracy # Graphics profiler
                vulkan-tools-lunarg # vkconfig
              ];

            # build and runtime dependencies 
            buildInputs = with pkgs; [ stdenv.cc.cc.lib ];
            # runtime packages
            packages = with pkgs; [
              xorg.libXcursor
              xorg.libXrandr
              xorg.libXi
              wayland
              wayland-protocols
              wayland-utils
              libxkbcommon
            ];
            CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER =
              lib.optional pkgs.stdenv.isLinux "${pkgs.clang}/bin/clang";
            RUSTFLAGS = lib.optional pkgs.stdenv.isLinux
              "-C link-arg=-fuse-ld=${pkgs.mold}/bin/mold";
            RUST_SRC_PATH = "${rustTools}/lib/rustlib/src/rust/library";
            LD_LIBRARY_PATH = with pkgs;
              "$LD_LIBRARY_PATH:${pkgs.stdenv.cc.cc.lib}/lib:${vulkan-loader}/lib:${vulkan-validation-layers}/lib:${wayland}/lib:${libxkbcommon}/lib:${shaderc.lib}/lib";
            VULKAN_SDK = with pkgs; "${vulkan-headers}";
            VK_LAYER_PATH = with pkgs;
              "${vulkan-validation-layers}/share/vulkan/explicit_layer.d";
            VULKAN_LIB_DIR = "${pkgs.shaderc.lib}/lib";
            SHADERC_LIB_DIR = with pkgs; "${shaderc.static}/lib";

            RUST_BACKTRACE = "full";
          };
          packages.default = pkgs.rustPlatform.buildRustPackage {
            pname = "my-rust-project";
            version = "0.1.0";
            src = ./.;

            installPhase = ''
              mkdir -p $out/bin
              cp target/release/my-rust-project $out/bin/
              echo "Cache will be stored in ~/.cache/my-rust-project"
            '';

            postUninstall = ''
              echo "Cleaning up cache files..."
              rm -rf ~/.cache/my-rust-project
            '';

            meta = with pkgs.lib; {
              description = "A Rust project with Vulkan and Wayland support";
              license = licenses.mit;
              platforms = platforms.linux ++ platforms.darwin;
            };
          };
        };
    };
}
