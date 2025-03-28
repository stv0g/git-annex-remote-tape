{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    naersk = {
      url = "github:nix-community/naersk/master";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    rustowl.url = "github:mrcjkb/rustowl-flake";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      fenix,
      naersk,
      rustowl,
    }:
    {
      overlays = {
        mhvtl = final: prev: {
          mhvtl = final.callPackage ./nix/mhvtl { };
          git-annex-remote-tape = final.callPackage ./nix { inherit naersk; };

          linuxPackages = prev.linuxPackages.extend (_: _: { mhvtl = final.mhvtl.linuxPackage; });
        };
      };

      nixosModules = {
        mhvtl = import ./nix/mhvtl/module.nix;
      };
    }
    // flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            self.overlays.mhvtl
            fenix.overlays.default
            rustowl.overlays.default
          ];
        };
      in
      {
        packages = rec {
          default = git-annex-remote-tape;

          inherit (pkgs) mhvtl git-annex-remote-tape;
        };

        checks = rec {
          default = git-annex-remote-tape;

          git-annex-remote-tape = pkgs.callPackage ./nix/test { inherit self; };
          mhvtl = pkgs.callPackage ./nix/mhvtl/test.nix { inherit self; };
        };

        devShells.default =
          let
            rustToolchain =
              with pkgs.fenix;
              combine [
                targets.x86_64-unknown-linux-gnu.stable.rust-std
                stable.rust-src
                stable.rustc
                stable.cargo
                stable.rustfmt
              ];
          in
          pkgs.mkShell {
            buildInputs = [
              rustToolchain
              pkgs.rustowl
            ];

            RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library/";
            RUST_BACKTRACE = "1";
          };

        formatter = pkgs.nixfmt-rfc-style;
      }
    );
}
