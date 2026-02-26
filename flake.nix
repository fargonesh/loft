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

  outputs = { self, nixpkgs, flake-utils, rust-overlay, devenv } @ inputs:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        
        rustToolchain = pkgs.rust-bin.nightly.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
          targets = [ "wasm32-unknown-unknown" ];
        };

        loft = pkgs.rustPlatform.buildRustPackage {
          pname = "loft";
          version = "0.1.0";

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = with pkgs; [
            pkg-config
            mdbook
          ];

          buildInputs = with pkgs; [
            openssl
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.darwin.apple_sdk.frameworks.Security
            pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
          ];

          # Build both binaries
          cargoBuildFlags = [ "--bin" "loft" "--bin" "loft-lsp" ];

          postInstall = ''
            # Generate the mdBook documentation
            echo "Building loft book..."
            mkdir -p $out/share/loft
            mdbook build book --dest-dir $out/share/loft/book

            # Generate stdlib documentation
            echo "Generating stdlib docs..."
            $out/bin/loft stdlib-doc --output $out/share/loft/stdlib-docs
          '';

          meta = with pkgs.lib; {
            description = "A modern, interpreted programming language with LSP support";
            homepage = "https://github.com/fargonesh/loft";
            license = licenses.mit;
            maintainers = [ ];
          };
        };

      in
      {
        packages = {
          default = loft;
          loft = loft;
        };

        apps = {
          default = flake-utils.lib.mkApp {
            drv = loft;
            exePath = "/bin/loft";
          };
          loft = flake-utils.lib.mkApp {
            drv = loft;
            exePath = "/bin/loft";
          };
          loft-lsp = flake-utils.lib.mkApp {
            drv = loft;
            exePath = "/bin/loft-lsp";
          };
          serve = {
            type = "app";
            program = "${(pkgs.writeShellScriptBin "loft-serve" ''
              export PATH="${pkgs.nodejs}/bin:${rustToolchain}/bin:${pkgs.pkg-config}/bin:${pkgs.git}/bin:${pkgs.wasm-pack}/bin:$PATH"
              export PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig"
              export OPENSSL_DIR="${pkgs.openssl.dev}"
              export OPENSSL_LIB_DIR="${pkgs.openssl.out}/lib"
              export OPENSSL_INCLUDE_DIR="${pkgs.openssl.dev}/include"
              
              if [ ! -d "registry" ] || [ ! -d "www" ]; then
                echo "Error: Must be run from the root of the loft repository"
                exit 1
              fi

              echo "Starting Loft Package Registry..."
              echo "Starting Loft Package Registry..." >> /tmp/loft-serve.log
              
              if [ -d "src/wasm" ]; then
                echo "Building WASM runtime..."
                (cd src/wasm && wasm-pack build --target web --out-dir ../../www/src/wasm) >/dev/null 2>&1 || echo "WASM build failed, check logs"
              fi
              
              (cd registry && cargo run) &
              REG_PID=$!
              
              echo "Building WASM runtime..."
              # Try to build WASM if wasm-pack is available or if user has it
              if command -v wasm-pack >/dev/null 2>&1; then
                 (cd src/wasm && wasm-pack build --target web --out-dir ../../www/src/wasm) || echo "WASM build failed, playground may not work"
              else
                 echo "wasm-pack not found, skipping WASM build"
              fi

              echo "Starting Loft Web Server..."
              (cd www && npm install && npm run dev) &
              WEB_PID=$!
              
              trap "kill $REG_PID $WEB_PID" EXIT
              wait
            '')}/bin/loft-serve";
          };
        };

        devShells.default = devenv.lib.mkShell {
          inherit inputs pkgs;
          modules = [
            ./devenv.nix
            {
               # Ensure our custom rust toolchain is used if needed, 
               # but devenv handles rust nightly well already.
               # For now, let's just make it a clean shell.
            }
          ];
        };
      }
    );
}
