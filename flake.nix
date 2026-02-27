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
      nixosModule =
        {
          config,
          pkgs,
          lib,
          ...
        }:
        let
          cfg = config.services.loft;

          pkgs' = import nixpkgs {
            inherit (pkgs) system;
            overlays = [ (import rust-overlay) ];
          };

          rustToolchain = pkgs'.rust-bin.nightly.latest.default.override {
            extensions = [
              "rust-src"
              "rust-analyzer"
            ];
            targets = [ "wasm32-unknown-unknown" ];
          };

          loftPkg = self.packages.${pkgs.system}.default;

          runtimeDeps = with pkgs'; [
            bash
            coreutils
            nodejs
            nodePackages.npm
            git
            pkg-config
            wasm-pack
            openssl
            rustToolchain
            stdenv.cc
            gcc
            binutils
          ];

          serveScript = pkgs'.writeShellScriptBin "loft-serve" ''
            export PATH="${lib.makeBinPath runtimeDeps}:$PATH"
            export SHELL="${pkgs'.bash}/bin/bash"
            export CC="cc"

            export PKG_CONFIG_PATH="${pkgs'.openssl.dev}/lib/pkgconfig"
            export OPENSSL_DIR="${pkgs'.openssl.dev}"
            export OPENSSL_LIB_DIR="${pkgs'.openssl.out}/lib"
            export OPENSSL_INCLUDE_DIR="${pkgs'.openssl.dev}/include"

            export GITHUB_CLIENT_ID="${cfg.githubClientId}"
            export GITHUB_CLIENT_SECRET="${cfg.githubClientSecret}"
            export PUBLIC_URL="${cfg.publicUrl}"
            export FRONTEND_URL="${cfg.frontendUrl}"
            export STORAGE_DIR="${cfg.storageDir}"
            export JWT_SECRET="${cfg.jwtSecret}"
            export BIND_ADDR="${cfg.bindAddr}"

            WORKDIR="/var/lib/loft/src"
            mkdir -p "$WORKDIR"
            cp -af ${self}/. "$WORKDIR/"
            chmod -R u+w "$WORKDIR"
            cd "$WORKDIR"

            if [ -d "src/wasm" ]; then
              (cd src/wasm && wasm-pack build --target web --out-dir ../../www/src/wasm) || echo "WASM build failed"
            fi

            ${loftPkg}/bin/loft stdlib-doc --output www/public/docs/stdlib || echo "Docs failed"

            # Note: We keep the cp of book/src if you still use those markdown files
            # for your frontend, but we no longer call mdbook.
            if [ -d "book/src" ]; then
                mkdir -p www/public/docs
                cp -r book/src/. www/public/docs/
            fi

            (cd registry && cargo run) &
            (cd www && npm install --ignore-scripts && npx vite --host 127.0.0.1 --port 9916 --cors --strictPort --config vite.config.js) &            
            wait
          '';
        in
        {
          options.services.loft = {
            enable = lib.mkEnableOption "Loft service";
            domain = lib.mkOption {
              type = lib.types.str;
              default = "loft.fargone.sh";
            };
            githubClientId = lib.mkOption { type = lib.types.str; };
            githubClientSecret = lib.mkOption { type = lib.types.str; };
            jwtSecret = lib.mkOption { type = lib.types.str; };
            publicUrl = lib.mkOption {
              type = lib.types.str;
              default = "https://registry.loft.fargone.sh";
            };
            frontendUrl = lib.mkOption {
              type = lib.types.str;
              default = "https://loft.fargone.sh";
            };
            storageDir = lib.mkOption {
              type = lib.types.str;
              default = "/var/lib/loft/registry-storage";
            };
            bindAddr = lib.mkOption {
              type = lib.types.str;
              default = "0.0.0.0:5050";
            };
            frontendPort = lib.mkOption {
              type = lib.types.str;
              default = "127.0.0.1:9916";
            };
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
            services.nginx.virtualHosts."${cfg.domain}" = {
              useACMEHost = "fargone.sh";
              forceSSL = true;
              locations."/" = {
                proxyPass = "http://${cfg.frontendPort}";
                proxyWebsockets = true;
                # Force the Host header to match the domain Vite is expecting
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

        loft = pkgs.rustPlatform.buildRustPackage {
          pname = "loft";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          nativeBuildInputs = with pkgs; [ pkg-config ]; # Removed mdbook
          buildInputs = with pkgs; [ openssl ];
          cargoBuildFlags = [
            "--bin"
            "loft"
            "--bin"
            "loft-lsp"
          ];

          postInstall = ''
            mkdir -p $out/share/loft
            # Only generate stdlib docs during the build phase
            $out/bin/loft stdlib-doc --output $out/share/loft/stdlib-docs || true
          '';
        };
      in
      {
        packages.default = loft;
        devShells.default = inputs.devenv.lib.mkShell {
          inherit inputs pkgs;
          modules = [ ./devenv.nix ];
        };
      }
    );
}
