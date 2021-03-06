{ post-build-postgres }:
{ lib, pkgs, config, ... }:
with lib;
let
  cfg = config.services.post-build-postgres;
in
{
  options.services.post-build-postgres = {
    enable = mkEnableOption "post-build-hook postgres service";

    databaseUrlScript = mkOption {
      type = types.str;
      example = "${pkgs.awscli2}/bin/aws secretsmanager get-secret-value --secret-id database-url --region us-east-1 | ${pkgs.jq}/bin/jq -r .SecretString |  ${pkgs.jq}/bin/jq -r .DATABASE_URL";
    };
  };

  config = mkIf cfg.enable {
    systemd.services.post-build-postgres = {
      wantedBy = [ "multi-user.target" ];
      after = [ "network-online.target" ];
      serviceConfig =
        {
          ExecStart = "/bin/sh -c 'export DATABASE_URL=$(${cfg.databaseUrlScript}); exec ${post-build-postgres}/bin/post-build-upload'";
          Restart = "always";
          RestartSec = 10;
        };
    };

    nix.extraOptions = ''
      post-build-hook = ${post-build-postgres}/bin/post-build-hook
    '';
  };
}
