{ self, testers }:
testers.runNixOSTest (
  { lib, config, ... }:
  let
    inherit (builtins) toJSON;
    inherit (config.nodes) machine;
  in
  {
    name = "mhvtl";

    extraPythonPackages = p: [ p.dotmap ];
    skipTypeCheck = true;

    nodes.machine =
      { pkgs, ... }:
      {
        imports = [ self.nixosModules.mhvtl ];

        environment.systemPackages = with pkgs; [
          mtx
          mt-st
          lsscsi
          mhvtl
          kmod
        ];

        hardware.mhvtl = {
          enable = true;

          drives = map (i: {
            id = i;
            vendor = "IBM";
            product = "ULT3580-TD6";
            revision = "2160";
            serial = "SN00000${toString i}";
            naa = "10:22:33:44:ab:cd:ef:0${toString i}";
            library = {
              id = 10;
              slot = i;
            };
          }) (lib.range 1 2);
        };
      };

    testScript = ''
      import json
      from dotmap import DotMap

      def get_file_contents(file):
        return machine.succeed(f"cat {file}").strip()

      def get_scsi_host_id():
        out = machine.succeed("ls -1 /sys/bus/mhvtl/drivers/mhvtl/adapter0/")
        for line in out.split("\n"):
          if line.startswith("host"):
            return int(line.removeprefix("host"))

      def run(cmd):
        print(machine.succeed(cmd))

      machine.wait_for_unit("mhvtl.target")

      with subtest("Check that kernel module is loaded"):
        run("modinfo mhvtl")

      with subtest("Get id of SCSI host"):
        host = get_scsi_host_id()
        run(f"lsscsi -lll {host}")

      files = ["Hello World", "Hallo Welt", "Hola Mundo"]
      drives = [DotMap(d) for d in json.loads('${toJSON machine.hardware.mhvtl.drives}')]

      for drive in drives:
        drive.st = f"/dev/tape/by-id/scsi-{drive.serial}"
        drive.nst = drive.st + "-nst"
        drive.sysfs = f"/sys/bus/scsi/devices/{host}:{drive.scsi.channel}:{drive.scsi.target}:{drive.scsi.lun}"

      with subtest("Wait for devices to appear"):
        for drive in drives:
          machine.wait_for_file(drive.nst);
        machine.wait_for_file("/dev/sch0");

      with subtest("Check that devices are created"):
        for drive in drives:
          assert get_file_contents(drive.sysfs + "/vendor") == drive.vendor
          assert get_file_contents(drive.sysfs + "/model") == drive.product

        assert get_file_contents(f"/sys/bus/scsi/devices/{host}:0:0:10/vendor") == "SCALAR"
        assert get_file_contents(f"/sys/bus/scsi/devices/{host}:0:0:10/model") == "i40L"

      with subtest("Initialize a new tape"):
        run("mktape -s 2500 -d LTO6 -l 10 -t data -m LTO001L6")

      with subtest("Load tape into drive 0"):
        run("mtx -f /dev/sch0 load 1 0")
        run("mtx -f /dev/sch0 status")
        run(f"mt -f {drives[0].nst} status")

      with subtest("Write files into onto tape"):
        run(f"mt -f {drives[0].nst} rewind")
        for file in files:
          run(f"echo '{file}' > {drives[0].nst}")

      with subtest("Check that files are on tape"):
        run(f"mt -f {drives[0].nst} rewind")
        for file in files:
          assert get_file_contents(drives[0].nst) == file

      with subtest("Move tape from drive 0 to 1"):
        run("mtx -f /dev/sch0 unload 1")
        run("mtx -f /dev/sch0 load 1 1")

      with subtest("Check that files are still on tape using new drive"):
        run(f"mt -f {drives[1].nst} rewind")
        for file in files:
          assert get_file_contents(drives[1].nst) == file
    '';

    meta = {
      maintainers = with lib.maintainers; [ stv0g ];
      platforms = [ "x86_64-linux" ];
    };
  }
)
