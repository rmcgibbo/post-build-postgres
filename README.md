# post-build-postgres
Nix post-build-hook to upload to postgresql

Usage:
```
{ config, lib, ... }:
let
  post-build-postgres = builtins.fetchTarball {
    url = "https://github.com/rmcgibbo/post-build-postgres/archive/81142c130fc678941d42f6bc70a234854422d9a3.tar.gz";
    sha256 = "112rrhrwkl5xxsvjnixiws6iridmmlzjvxwhrvcp0q6j69326pf7";
  };
in {
  imports = [
    "${post-build-postgres}/module.nix"
    ...
  ];

  services.post-build-postgres = {
    enable = true;
  };
  
  ...
}
```
