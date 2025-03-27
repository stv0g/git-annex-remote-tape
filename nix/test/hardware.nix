{ self, lib, ... }:
let
  inherit (lib) replicate;
in
{
  imports = [ self.nixosModules.mhvtl ];

  hardware.mhvtl = {
    enable = true;

    drives =
      map
        (i: {
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
        })
        [
          1
          2
        ];

    libraries = [
      {
        id = 10;
        vendor = "SCALAR";
        product = "i40";
        revision = "2160";
        serial = "D0H0029914";
        naa = "10:22:33:44:ab:cd:ef:00";

        slots = map (i: "LTO00${toString i}L6") (lib.range 1 8);
        maps = replicate 2 "";
        drives = replicate 4 "";
        pickers = replicate 1 "";
      }
    ];
  };
}
