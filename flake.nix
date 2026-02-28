{
  description = "loft - A modern, interpreted programming language";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    devenv.url = "github:cachix/devenv";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
      ...
    }@inputs:
    let
      # --- Helper function to create the serve script ---
      # This allows us to share the logic between the NixOS module and the Flake app
      makeServeScript = pkgs: lib: loftPkg: rustToolchain: cfg: 
        let
          runtimeDeps = with pkgs; [
            bash coreutils nodejs nodePackages.npm git pkg-config
            wasm-pack openssl rustToolchain stdenv.cc gcc binutils
          ];
        in
        pkgs.writeShellScriptBin "loft-serve" ''
          export PATH="${lib.makeBinPath runtimeDeps}:$PATH"
          export SHELL="${pkgs.bash}/bin/bash"
          export CC="cc"

          export PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig"
          export OPENSSL_DIR="${pkgs.openssl.dev}"
          export OPENSSL_LIB_DIR="${pkgs.openssl.out}/lib"
          export OPENSSL_INCLUDE_DIR="${pkgs.openssl.dev}/include"

          # Fallback to defaults if env vars aren't set (useful for nix run)
          export GITHUB_CLIENT_ID="''${GITHUB_CLIENT_ID:-${cfg.githubClientId}}"
          export GITHUB_CLIENT_SECRET="''${GITHUB_CLIENT_SECRET:-${cfg.githubClientSecret}}"
          export PUBLIC_URL="''${PUBLIC_URL:-${cfg.publicUrl}}"
          export FRONTEND_URL="''${FRONTEND_URL:-${cfg.frontendUrl}}"
          export STORAGE_DIR="''${STORAGE_DIR:-${cfg.storageDir}}"
          export JWT_SECRET="''${JWT_SECRET:-${cfg.jwtSecret}}"
          export BIND_ADDR="''${BIND_ADDR:-${cfg.bindAddr}}"

          WORKDIR="''${LOFT_WORKDIR:-/tmp/loft-run}"
          mkdir -p "$WORKDIR"
          cp -af ${self}/. "$WORKDIR/"
          chmod -R u+w "$WORKDIR"
          cd "$WORKDIR"

          if [ -d "src/wasm" ]; then
            (cd src/wasm && wasm-pack build --target web --out-dir ../../www/src/wasm) || echo "WASM build failed"
          fi

          ${loftPkg}/bin/loft stdlib-doc --output www/public/docs/stdlib || echo "Docs failed"

          if [ -d "book/src" ]; then
              mkdir -p www/public/docs
              cp -r book/src/. www/public/docs/
          fi

          (cd registry && cargo run) &
          (cd www && npm install --ignore-scripts && npx vite --host 127.0.0.1 --port 9916 --cors --strictPort --config vite.config.js) &            
          wait
        '';

      # --- NixOS Module ---
      nixosModule = { config, pkgs, lib, ... }:
        let
          cfg = config.services.loft;
          pkgs' = import nixpkgs {
            inherit (pkgs) system;
            overlays = [ (import rust-overlay) ];
          };
          rustToolchain = pkgs'.rust-bin.nightly.latest.default.override {
            extensions = [ "rust-src" "rust-analyzer" ];
            targets = [ "wasm32-unknown-unknown" ];
          };
          loftPkg = self.packages.${pkgs.system}.default;
          
          # Call the shared script generator
          serveScript = makeServeScript pkgs' lib loftPkg rustToolchain cfg;
        in
        {
          options.services.loft = {
             enable = lib.mkEnableOption "Loft service";
             domain = lib.mkOption { type = lib.types.str; default = "loft.fargone.sh"; };
             githubClientId = lib.mkOption { type = lib.types.str; default = ""; };
             githubClientSecret = lib.mkOption { type = lib.types.str; default = ""; };
             jwtSecret = lib.mkOption { type = lib.types.str; default = "dev-secret"; };
             publicUrl = lib.mkOption { type = lib.types.str; default = "http://localhost:9916"; };
             frontendUrl = lib.mkOption { type = lib.types.str; default = "http://localhost:9916"; };
             storageDir = lib.mkOption { type = lib.types.str; default = "/var/lib/loft/registry-storage"; };
             bindAddr = lib.mkOption { type = lib.types.str; default = "0.0.0.0:5050"; };
             frontendPort = lib.mkOption { type = lib.types.str; default = "127.0.0.1:9916"; };
          };

          config = lib.mkIf cfg.enable {
            systemd.services.loft = {
              wantedBy = [ "multi-user.target" ];
              after = [ "network.target" ];
              serviceConfig = {
                ExecStart = "${serveScript}/bin/loft-serve";
                StateDirectory = "loft";
                User = "root";
                Restart = "on-failure";
              };
            };
            # ... Nginx config remains the same ...
            services.nginx.virtualHosts."${cfg.domain}" = {
              useACMEHost = "fargone.sh";
              forceSSL = true;
              locations."/" = {
                proxyPass = "http://${cfg.frontendPort}";
                proxyWebsockets = true;
                extraConfig = ''
                  proxy_set_header Host ${cfg.domain};
                  proxy_set_header X-Real-IP $remote_addr;
                  proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
                  proxy_set_header X-Forwarded-Proto $scheme;
                '';
              };
            };
          };
        };
    in
    {
      nixosModules.default = nixosModule;
    }
    // flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
        lib = pkgs.lib;

        rustToolchain = pkgs.rust-bin.nightly.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
          targets = [ "wasm32-unknown-unknown" ];
        };

        loft = pkgs.rustPlatform.buildRustPackage {
          pname = "loft";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          nativeBuildInputs = with pkgs; [ pkg-config ];
          buildInputs = with pkgs; [ openssl ];
          cargoBuildFlags = [ "--bin" "loft" "--bin" "loft-lsp" ];
          postInstall = ''
            mkdir -p $out/share/loft
            $out/bin/loft stdlib-doc --output $out/share/loft/stdlib-docs || true
          '';
        };

        # Default config for local "nix run"
        defaultServeCfg = {
          githubClientId = "";
          githubClientSecret = "";
          jwtSecret = "dev-secret-only";
          publicUrl = "http://localhost:5050";
          frontendUrl = "http://localhost:9916";
          storageDir = "./storage";
          bindAddr = "127.0.0.1:5050";
        };

        servePkg = makeServeScript pkgs lib loft rustToolchain defaultServeCfg;
      in
      {
        packages = {
          default = loft;
          serve = servePkg;
        };

        # This allows you to run: nix run .#serve
        apps.serve = flake-utils.lib.mkApp {
          drv = servePkg;
          name = "loft-serve";
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustToolchain
            rust-analyzer
            pkg-config
            openssl
            cargo-watch
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.darwin.apple_sdk.frameworks.Security
            pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
          ];

          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
          # Ensure persistent storage directory exists for registry
          shellHook = ''
            if [ ! -d /var/lib/loft-registry ]; then
              sudo mkdir -p /var/lib/loft-registry
              sudo chown "$USER" /var/lib/loft-registry
            fi
          '';
        };
      }
    );
}