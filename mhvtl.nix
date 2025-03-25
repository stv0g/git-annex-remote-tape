{
  lib,
  stdenv,
  fetchFromGitHub,
  zlib,
  getopt,
  perl,
  linuxPackages,
  kernel ? linuxPackages.kernel,
}:
let
  name = "mhvtl";
  version = "1.7-2";

  src = fetchFromGitHub {
    owner = "markh794";
    repo = "mhvtl";
    rev = "${version}_release";
    hash = "sha256-m7yRA9TJdQPHZa/RuOF9u1LaLmLidJBcjO6zRfsKlRI=";
    leaveDotGit = true;
  };

  linuxPackage = stdenv.mkDerivation {
    pname = "${name}-module";
    inherit version src;

    sourceRoot = "source/kernel";

    nativeBuildInputs = kernel.moduleBuildDependencies;

    makeFlags = kernel.makeFlags ++ [
      "KDIR=${kernel.dev}/lib/modules/${kernel.modDirVersion}/build"
      "V=${kernel.modDirVersion}"
    ];

    postPatch = ''
      patchShebangs --build config.sh
      substituteInPlace Makefile --replace "mhvtl.ko /lib/modules" "mhvtl.ko $out/lib/modules"
    '';

    preInstall = ''
      mkdir -p $out/lib/modules/${kernel.modDirVersion}/kernel/drivers/scsi/
    '';

    buildFlags = [ "default" ];
    installTargets = [ "install" ];

    meta = {
      description = "A kernel module for the mhvtl Linux Virtual Tape Library";
      homepage = "https://github.com/markh794/mhvtl";
      license = lib.licenses.gpl2;
      maintainers = [ lib.maintainers.stv0g ];
      platforms = lib.platforms.linux;
    };
  };
in
stdenv.mkDerivation (finalAttrs: {
  pname = name;
  inherit version src;

  passthru = {
    inherit linuxPackage;
  };

  buildInputs = [
    zlib
    perl
  ];

  nativeBuildInputs = [ getopt ];

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

  separateDebugInfo = true;

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
    outputsToInstall = [
      "out"
      "debug"
    ];
  };
})
