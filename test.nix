{ self, testers }:
testers.runNixOSTest (
  { pkgs, lib, ... }:
  {
    name = "git-annex-remote-tape";

    extraPythonPackages = p: [ p.dotmap ];
    skipTypeCheck = true;
    node.specialArgs = {
      inherit self;
    };

    nodes.machine = {
      imports = [ ./mhvtl-hardware.nix ];
    };

    testScript =
      { nodes }:
      let
        pkgs = nodes.machine.config.nixpkgs.pkgs;
        naersk-lib = pkgs.callPackage self.inputs.naersk { };

        # Build test artifacts for execution in NixOS VM
        git-annex-remote-tape-test = naersk-lib.buildPackage {
          name = "git-annex-remote-tape";
          src = ./.;

          cargoBuild =
            _: ''cargo $cargo_options test --no-run $cargo_build_options >> $cargo_build_output_json'';
          copyBinsFilter = ''select(.reason == "compiler-artifact" and .executable != null and .profile.test == true)'';

          postInstall = ''
            mv $out/bin/git_annex_remote_tape-* $out/bin/unit_test
            mv $out/bin/integration_test-* $out/bin/integration_test
          '';
        };
      in
      ''
        machine.wait_for_unit("default.target")

        with subtest("Unit tests"):
          machine.execute("${git-annex-remote-tape-test}/bin/unit_test > /dev/console")

        with subtest("Integration tests"):
          machine.execute("${git-annex-remote-tape-test}/bin/integration_test > /dev/console")
      '';

    meta = {
      maintainers = with lib.maintainers; [ stv0g ];
      platforms = [ "x86_64-linux" ];
    };
  }
)
