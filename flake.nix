{
  description = "A Nix-flake-based Rust development environment";

  inputs = {
    nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0.1.*.tar.gz";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, rust-overlay }:
    let
      supportedSystems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forEachSupportedSystem = f: nixpkgs.lib.genAttrs supportedSystems (system: f {
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default self.overlays.default ];
        };
      });
    in
    {
      overlays.default = final: prev: {
        rustToolchain =
          let
            rust = prev.rust-bin;
          in
          if builtins.pathExists ./rust-toolchain.toml then
            rust.fromRustupToolchainFile ./rust-toolchain.toml
          else if builtins.pathExists ./rust-toolchain then
            rust.fromRustupToolchainFile ./rust-toolchain
          else
            rust.stable.latest.default.override {
              extensions = [ "rust-src" "rustfmt" ];
            };
      };

      devShells = forEachSupportedSystem ({ pkgs }:
        let libPath = with pkgs; lib.makeLibraryPath [
              # stolen from: https://github.com/gfx-rs/wgpu/blob/trunk/shell.nix
            ];
        in
        {
        default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustToolchain
            openssl
            pkg-config
            cargo-deny
            cargo-edit
            cargo-watch
            rust-analyzer
            cargo-bootimage
            qemu

            # nice tools
            gdb rr
            evcxr
            valgrind
            renderdoc
          ];

          env = {
            # Required by rust-analyzer
            RUST_SRC_PATH = "${pkgs.rustToolchain}/lib/rustlib/src/rust/library";
            LD_LIBRARY_PATH = libPath;
          };
        };
      });
    };
}
