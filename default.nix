{
  pkgs,
  pkg-config,
  openssl,
  systemd
}:
let
  naersk = pkgs.callPackage (pkgs.fetchFromGitHub {
    owner = "nmattia";
    repo = "naersk";
    rev = "e0fe990b478a66178a58c69cf53daec0478ca6f9";
    sha256 = "0qjyfmw5v7s6ynjns4a61vlyj9cghj7vbpgrp9147ngb1f8krz2c";
  }) { };
in naersk.buildPackage {
  root = ./.;

  nativeBuildInputs = [
    pkg-config
  ];
  buildInputs = [
    openssl
    systemd
  ];
}
