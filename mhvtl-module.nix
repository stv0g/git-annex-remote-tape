{
  lib,
  pkgs,
  config,
  ...
}:
let
  cfg = config.hardware.mhvtl;

  inherit (lib) boolToString optionalString imap1;
  inherit (builtins)
    concatStringsSep
    map
    toString
    listToAttrs
    ;

  scsi_addr = lib.types.submodule {
    options = {
      channel = lib.mkOption {
        description = "SCSI channel";
        type = lib.types.int;
      };

      target = lib.mkOption {
        description = "SCSI target";
        type = lib.types.int;
      };

      lun = lib.mkOption {
        description = "SCSI LUN";
        type = lib.types.int;
      };
    };
  };

  drive = lib.types.submodule (
    { config, ... }:
    {
      options = {
        id = lib.mkOption {
          description = "Queue ID";
          type = lib.types.int;
        };

        scsi = lib.mkOption {
          description = "SCSI address";
          type = scsi_addr;
          default = {
            channel = 0;
            target = 0;
            lun = config.id;
          };
        };

        library = {
          id = lib.mkOption {
            description = "Library ID";
            type = lib.types.int;
          };
          slot = lib.mkOption {
            description = "Library slot";
            type = lib.types.int;
          };
        };

        vendor = lib.mkOption {
          description = "Vendor identification";
          type = lib.types.str;
        };

        product = lib.mkOption {
          description = "Product identification";
          type = lib.types.str;
        };

        revision = lib.mkOption {
          description = "Product revision level";
          type = lib.types.str;
        };

        serial = lib.mkOption {
          description = "Unit serial number";
          type = lib.types.str;
        };

        naa = lib.mkOption {
          description = "Network Address Authority (NAA)";
          type = lib.types.nullOr lib.types.str;
        };

        backoff = lib.mkOption {
          description = "Additional delay in uS";
          type = lib.types.ints.between 10 10000;
          default = 1000;
        };

        fifo = lib.mkOption {
          description = "Send near real time state information for external applications to a named pipe";
          type = lib.types.nullOr lib.types.path;
          default = null;
        };

        compression = {
          enable = lib.mkEnableOption "Enable compression";

          factor = lib.mkOption {
            type = lib.types.ints.between 1 9;
            default = 9;
          };

          type = lib.mkOption {
            type = lib.types.enum [
              "lzo"
              "zlib"
            ];
            default = "zlib";
          };
        };
      };
    }
  );

  library = lib.types.submodule (
    { config, ... }:
    {
      options = {
        id = lib.mkOption {
          description = "Queue / Device ID";
          type = lib.types.int;
        };

        scsi = lib.mkOption {
          description = "SCSI address";
          type = scsi_addr;
          default = {
            channel = 0;
            target = 0;
            lun = config.id;
          };
        };

        vendor = lib.mkOption {
          description = "Vendor identification";
          type = lib.types.str;
        };

        product = lib.mkOption {
          description = "Product identification";
          type = lib.types.str;
        };

        revision = lib.mkOption {
          description = "Product revision level";
          type = lib.types.str;
        };

        serial = lib.mkOption {
          description = "Unit serial number";
          type = lib.types.str;
        };

        naa = lib.mkOption {
          description = "Network Address Authority (NAA)";
          type = lib.types.nullOr lib.types.str;
        };

        backoff = lib.mkOption {
          description = "Additional delay in uS";
          type = lib.types.ints.between 10 10000;
          default = 1000;
        };

        fifo = lib.mkOption {
          description = "Send near real time state information for external applications to a named pipe";
          type = lib.types.nullOr lib.types.path;
          default = null;
        };

        homePath = lib.mkOption {
          description = "Specify a parent directory for the virtual media associated with this library";
          type = lib.types.path;
          default = "${cfg.defaultHomePath}/${toString config.id}";
        };

        persist = lib.mkOption {
          description = ''
            Enable saving state on shutdown.
            A file 'library_contents.XX.persist' will be created on shutdown to save persistent state.
            On startup - Default to read the library_contents.XX.persist if the file exists. Fall back to library_contents.XX if no .persist file exists.
          '';

          type = lib.types.bool;
          default = false;
        };

        drives = lib.mkOption {
          description = "A list of barcodes of media loaded to drives. Use an empty string for empty drives.";
          type = lib.types.listOf lib.types.str;
          default = [ ];
        };

        pickers = lib.mkOption {
          description = "A list of barcodes of media currently picked. Use an empty string for empty pickers.";
          type = lib.types.listOf lib.types.str;
          default = [ ];
        };

        maps = lib.mkOption {
          description = "A list of barcodes of media loaded to media access ports (MAPS). Use an empty string for empty MAPS.";
          type = lib.types.listOf lib.types.str;
          default = [
            ""
          ];
        };

        slots = lib.mkOption {
          description = "A list of barcodes of media placed into storage slots. Use an empty string for empty slots.";
          type = lib.types.listOf lib.types.str;
          default = [
            "LTO001L6"
            "LTO002L6"
            "LTO003L6"
          ];
        };
      };
    }
  );

  commonTemplate =
    type: x:
    ''
      ${type}: ${toString x.id} CHANNEL: ${toString x.scsi.channel} TARGET: ${toString x.scsi.target} LUN: ${toString x.scsi.lun}
        Vendor identification: ${x.vendor}
        Product identification: ${x.product}
        Product revision level: ${x.revision}
        Unit serial number: ${x.serial}
        NAA: ${x.naa}
        Backoff: ${toString x.backoff}''
    + (optionalString (x.fifo != null) "\n fifo: ${x.fifo}");

  library_contents =
    let
      entries = name: entries: imap1 (i: s: "${name} ${toString i}: ${s}") entries;
    in
    l: ''
      VERSION 2

      ${concatStringsSep "\n" (
        (entries "Drive" l.drives)
        ++ (entries "Slot" l.slots)
        ++ (entries "Picker" l.pickers)
        ++ (entries "MAP" l.maps)
      )}
    '';

  device_conf =
    ''
      VERSION 5

    ''
    + concatStringsSep "\n" (
      (map (l: ''
        ${commonTemplate "Library" l}
          Home directory: ${l.homePath}
          PERSIST: ${boolToString l.persist}
      '') cfg.libraries)
      ++ (map (d: ''
        ${commonTemplate "Drive" d}
          Library ID: ${toString d.library.id} Slot: ${toString d.library.slot}
          Compression: factor ${toString d.compression.factor} enabled ${toString d.compression.enable}
          Compression type: ${d.compression.type}
      '') cfg.drives)
    );
in
{
  options = {
    hardware.mhvtl = {
      enable = lib.mkEnableOption "Linux Virtual Tape Library";

      package = lib.mkPackageOption pkgs "mhvtl" { };

      kernelPackage = lib.mkPackageOption pkgs [
        "linuxPackages"
        "mhvtl"
      ] { };

      drives = lib.mkOption {
        description = "List of tape drives";
        type = lib.types.listOf drive;
        default = [
          {
            id = 1;
            vendor = "IBM";
            product = "ULTRIUM-HH6";
            revision = "E4J1";
            serial = "MYK0830KS2";
            naa = "10:22:33:44:ab:cd:ef:01";
            library = {
              id = 10;
              slot = 1;
            };
          }
        ];
      };

      libraries = lib.mkOption {
        description = "List of tape changer libraries";
        type = lib.types.listOf library;
        default = [
          {
            id = 10;
            vendor = "SCALAR";
            product = "i40";
            revision = "5500";
            serial = "D0H0029914";
            naa = "10:22:33:44:ab:cd:ef:00";

            slots = map (i: "LTO00${toString i}L6") (lib.range 1 8);
            maps = lib.replicate 2 "";
            drives = lib.replicate 4 "";
            pickers = lib.replicate 1 "";
          }
        ];
      };

      homePath = lib.mkOption {
        type = lib.types.path;
        description = "Data directory for virtual tapes";
        default = "/var/lib/mhvtl";
      };

      verbose = lib.mkOption {
        type = lib.types.int;
        description = "Default verbosity";
        default = 1;
      };

      debug = {
        kernel = lib.mkOption {
          type = lib.types.bool;
          description = "Set kernel module debugging";
          default = false;
        };

        daemon = lib.mkOption {
          type = lib.types.bool;
          description = "Set daemon debugging";
          default = false;
        };
      };
    };
  };

  config = lib.mkIf cfg.enable {
    boot = {
      kernelModules = [
        "mhvtl"
        "sg"
      ];

      extraModprobeConfig = ''
        options mhvtl opts=${if cfg.debug.kernel then "1" else "0"}
      '';

      extraModulePackages = [ cfg.kernelPackage ];
    };

    environment = {
      systemPackages = [
        cfg.package
        pkgs.lsscsi
      ];

      etc =
        {
          "mhvtl/device.conf".text = device_conf;
        }
        // (listToAttrs (
          map (l: {
            name = "mhvtl/library_contents.${toString l.id}";
            value.text = library_contents l;
          }) cfg.libraries
        ));
    };

    systemd = {
      services = {
        "vtltape@" = {
          description = "Tape Daemon for Virtual Tape & Robot Library";
          documentation = [
            "man:vtltape(1)"
            "man:vtlcmd(1)"
          ];
          requires = [ "systemd-modules-load.service" ];
          after = [ "systemd-modules-load.service" ];
          before = [ "mhvtl.target" ];
          partOf = [ "mhvtl.target" ];

          serviceConfig = {
            Type = "simple";
            ExecStart = "${cfg.package}/bin/vtltape -F -q%i -v${builtins.toString cfg.verbose} ${lib.optionalString cfg.debug.daemon "-d"}";
            ExecStop = "${cfg.package}/bin/vtlcmd %i exit";
            KillMode = "none";
          };
        };

        "vtllibrary@" = {
          description = "Robot Library Daemon for Virtual Tape & Robot Library";
          documentation = [
            "man:vtllibrary(1)"
            "man:vtlcmd(1)"
          ];
          requires = [ "systemd-modules-load.service" ];
          after = [ "systemd-modules-load.service" ];
          before = [ "mhvtl.target" ];
          partOf = [ "mhvtl.target" ];

          serviceConfig = {
            Type = "simple";
            ExecStart = "${cfg.package}/bin/vtllibrary -F -q%i -v${builtins.toString cfg.verbose} ${lib.optionalString cfg.debug.daemon "-d"}";
            ExecStop = "${cfg.package}/bin/vtlcmd %i exit";
            ExecReload = "/usr/bin/kill -HUP \${MAINPID}";
            KillMode = "none";
          };
        };
      };

      targets = {
        mhvtl = {
          description = "mhvtl service allowing to start/stop all vtltape@.service and vtllibrary@.service instances at once";
          documentation = [
            "man:man:vtltape(1)"
            "man:man:vtllibrary(1)"
          ];
          wantedBy = [ "multi-user.target" ];
          wants =
            (map (d: "vtltape@${toString d.id}.service") cfg.drives)
            ++ (map (l: "vtllibrary@${toString l.id}.service") cfg.libraries);
        };
      };

      tmpfiles.rules = map (l: "d ${l.homePath} 0700 root - - -") cfg.libraries;
    };
  };
}
