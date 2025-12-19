{
  pkgs,
  lib,
  config,
  inputs,
  ...
}:

{
  env.GREET = "lang";
  packages = [
    pkgs.git
    pkgs.lld
    pkgs.mold
    pkgs.rust-analyzer
    pkgs.tree-sitter
    pkgs.devenv
    pkgs.nodejs
    pkgs.openssl
  ];

  languages.rust = {
    enable = true;
    channel = "nightly";
  };

  enterShell = ''
    rustupdate
    git --version
  '';
  
}
