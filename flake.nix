{
  description = "Zig dev environment";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  inputs.flake-utils.url = "github:numtide/flake-utils";

  inputs.zls = {
    url = "github:zigtools/zls";
    inputs.nixpkgs.follows = "nixpkgs";
  };

  inputs.rust-overlay = {
    url = "github:oxalica/rust-overlay";
    inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { nixpkgs, rust-overlay, flake-utils, ... }:
      flake-utils.lib.eachDefaultSystem (system:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs {
            inherit system overlays;
          };
        in
        {
          devShells.default = with pkgs; mkShell rec {
            buildInputs = [
              rust-bin.beta.latest.default

              libxkbcommon
              libGL
              wayland
            ];
            LD_LIBRARY_PATH = "${lib.makeLibraryPath buildInputs}";
            };
          }
        );
  }
