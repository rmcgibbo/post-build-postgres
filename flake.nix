{
  description = "Nix post-build-hook to upload to postgres";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    utils.url = "github:numtide/flake-utils";
    naersk = {
      url = "github:nmattia/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, utils, naersk }:
    # darwin is not supported because we depend on systemd
    utils.lib.eachSystem [ "aarch64-linux" "i686-linux" "x86_64-linux" ] (system:
      let
        pkgs = import nixpkgs { inherit system; };
        naersk-lib = pkgs.callPackage naersk { };
        pkg = naersk-lib.buildPackage {
          root = ./.;

          nativeBuildInputs = with pkgs; [
            pkg-config
          ];
          buildInputs = with pkgs; [
            openssl
            systemd
          ];
        };
      in {
        packages.post-build-postgres = pkg;
        defaultPackage = pkg;
        nixosModules.post-build-postgres = (import ./module.nix) { post-build-postgres = pkg; };
      });
}

