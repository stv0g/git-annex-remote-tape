{
  lib,
  stdenv,
  callPackage,
  fetchFromGitHub,
  zlib,
  getopt,
  perl,
  git,
}:
let
  name = "mhvtl";
  version = "1.7-2";

  src = fetchFromGitHub {
    owner = "markh794";
    repo = "mhvtl";
    rev = "${version}_release";
    hash = "sha256-ymDvkWvtbpCrCjub2wDjWrRgL6or9jvBwcb1gztYlgM=";
    leaveDotGit = true;
  };

  linuxPackage = callPackage ./kernel.nix { inherit version src; };

  package = stdenv.mkDerivation (finalAttrs: {
    pname = name;
    inherit version src;

    passthru = {
      inherit linuxPackage;
    };

    buildInputs = [
      zlib
      perl
    ];

    nativeBuildInputs = [
      getopt
      git
    ];

    makeFlags = [
      "PREFIX="
      "DESTDIR=$(out)"
      "LIBDIR=/lib"
      "FIRMWAREDIR=/firmware"
      "SYSTEMD_SERVICE_DIR=/lib/systemd/system"
      "SYSTEMD_GENERATOR_DIR=/lib/systemd/system-generators"
      "VERSION=${finalAttrs.version}"
      "MHVTL_HOME_PATH=/var/lib/mhvtl"
    ];

    postPatch = ''
      substituteInPlace Makefile --replace "MAKE_VTL_MEDIA = usr/make_vtl_media" "MAKE_VTL_MEDIA = true"
    '';

    preBuild = ''
      make -C usr make_vtl_media
    '';

    postInstall = ''
      rm -r $out/{firmware,var}
      rm $out/bin/mhvtl_kernel_mod_build
    '';

    postFixup = ''
      patchShebangs --host $out/bin/update_device.conf
    '';

    hardeningDisable = [ "fortify" ];

    meta = {
      description = "A kernel module for the mhvtl Linux Virtual Tape Library";
      homepage = "https://github.com/markh794/mhvtl";
      license = lib.licenses.gpl2;
      maintainers = [ lib.maintainers.stv0g ];
      platforms = lib.platforms.linux;
    };
  });
in
package
