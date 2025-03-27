{ callPackage, naersk }:
let
  naersk-lib = callPackage naersk { };
in
naersk-lib.buildPackage {
  name = "git-annex-remote-tape";
  src = ./.;
}
