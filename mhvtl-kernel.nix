{
  lib,
  stdenv,
  version,
  src,
  linuxPackages,
  kernel ? linuxPackages.kernel,
}:
stdenv.mkDerivation {
  pname = "mhvtl-module";
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
}
