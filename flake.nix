{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    naersk = {
      url = "github:nix-community/naersk/master";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      naersk,
    }:
    {
      overlays = {
        mhvtl = final: prev: {
          mhvtl = final.callPackage ./mhvtl.nix { };
          linuxPackages = prev.linuxPackages.extend (_: _: { mhvtl = final.mhvtl.linuxPackage; });
        };
      };

      nixosModules = {
        mhvtl = import ./mhvtl-module.nix;
      };
    }
    // flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ self.overlays.mhvtl ];
        };
        naersk-lib = pkgs.callPackage naersk { };
      in
      {
        packages = rec {
          default = git-annex-remote-tape;
          git-annex-remote-tape = naersk-lib.buildPackage ./.;

          inherit (pkgs) mhvtl;
        };

        checks.mhvtl = pkgs.callPackage ./mhvtl-test.nix { inherit self; };

        devShells.default =
          with pkgs;
          mkShell {
            buildInputs = [
              cargo
              rustc
              rustfmt
              pre-commit
              rustPackages.clippy
            ];

            RUST_SRC_PATH = rustPlatform.rustLibSrc;
          };

        formatter = pkgs.nixfmt-rfc-style;
      }
    );
}
