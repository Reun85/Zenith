{
  description = "Test Rust project";

  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = {
    flake-parts,
    ...
  }@inputs:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = ["x86_64-linux" "x86_64-darwin"];
      perSystem = {
        config,
        lib,
        system,
        ...
      }: let
        pkgs = import inputs.nixpkgs {
          inherit system;
          overlays = [inputs.rust-overlay.overlays.default];
        };
        rustTools = (pkgs.rust-bin.fromRustupToolchainFile
          ./rust-toolchain.toml)
        .override {extensions = ["rust-analyzer" "rust-src"];};
      in {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = [pkgs.pkg-config];
          packages = with pkgs;
            [cmake python3 cargo-watch rustTools]
          ++
            # vulkan
          [
            clang
            vulkan-headers
            vulkan-loader
            vulkan-validation-layers
            vulkan-tools        # vulkaninfo
            shaderc             # GLSL to SPIRV compiler - glslc
            renderdoc           # Graphics debugger
            tracy               # Graphics profiler
            vulkan-tools-lunarg # vkconfig
          ]
            ++ lib.optional pkgs.stdenv.isDarwin pkgs.libiconv;

          CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER =
            lib.optional pkgs.stdenv.isLinux "${pkgs.clang}/bin/clang";
          RUSTFLAGS =
            lib.optional pkgs.stdenv.isLinux
            "-C link-arg=-fuse-ld=${pkgs.mold}/bin/mold";
          RUST_SRC_PATH = "${rustTools}/lib/rustlib/src/rust/library";

          LD_LIBRARY_PATH=with pkgs;"${vulkan-loader}/lib:${vulkan-validation-layers}/lib";
          VULKAN_SDK =  with pkgs;"${vulkan-headers}";
          VK_LAYER_PATH = with pkgs;"${vulkan-validation-layers}/share/vulkan/explicit_layer.d";


          };
      };
    };
}
