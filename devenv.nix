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
    targets = [ "wasm32-unknown-unknown" "x86_64-unknown-linux-gnu" ];
    channel = "nightly";
  };

  enterShell = ''
    rustupdate
    git --version
  '';
  
}
