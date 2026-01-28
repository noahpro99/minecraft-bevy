{
  description = "A simple flake for a Rust bevy project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      rust-overlay,
      ...
    }:

    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };
        buildInputs = with pkgs; [
          udev
          alsa-lib
          vulkan-loader
          libxkbcommon
          wayland
          libGL
          mold
          clang
        ];
      in
      {
        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            bashInteractive
            pkg-config
            (rust-bin.stable.latest.default.override {
              extensions = [ "rust-src" ];
            })
          ];
          inherit buildInputs;
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
        };
      }
    );
}
