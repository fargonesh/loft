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
    pkgs.pkg-config
    pkgs.wasm-pack
  ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
    pkgs.darwin.apple_sdk.frameworks.Security
    pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
  ];

  languages.rust = {
    enable = true;
    channel = "nightly";
    components = [ "rustc" "cargo" "clippy" "rustfmt" "rust-analyzer" "rust-src" ];
    targets = [ "wasm32-unknown-unknown" ];
  };

  # Set common rust environment variables 
  # env.RUST_SRC_PATH = lib.mkForce "${pkgs.rust-bin.nightly.latest.rust-src}/lib/rustlib/src/rust/library";

  env.OPENSSL_DIR = "${pkgs.openssl.dev}";
  env.OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
  env.OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include";

  languages.javascript = {
    enable = true;
    npm.enable = true;
  };

  processes = {
    www.exec = "cd www && npm install && npm run dev";
    registry.exec = "cd registry && cargo run";
  };

  enterShell = ''
    git --version
  '';
  
}
