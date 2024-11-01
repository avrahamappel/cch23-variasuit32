{
  description = "Rust Template";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        inherit (pkgs) lib stdenv;
        inherit (pkgs.darwin.apple_sdk) frameworks;
        inherit (pkgs.rustPlatform) buildRustPackage;

        cch23-validator = buildRustPackage rec {
          pname = "cch23-validator";
          version = "22.0.1";

          src = pkgs.fetchzip {
            url = "https://crates.io/api/v1/crates/${pname}/${version}/download";
            hash = "sha256-LLn1zo6N53bjzpQxOT6VyKN+9cDO516fen3EQf8fWDc=";
            extension = "tar";
          };

          cargoHash = "sha256-xxhc+wBP+Rr/+a2lbZsadpA+HnjZKQi7Ye2QiEv5aTU=";

          buildInputs = [
            (lib.optional stdenv.isDarwin frameworks.SystemConfiguration)
          ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            postgresql
            rust-bin.stable.latest.default
            rust-analyzer
            cargo-shuttle
            cargo-watch
            cch23-validator
            websocat # remember to use --linemode-strip-newlines so "\n" isn't passed back
          ];

          buildInputs = with pkgs; [ pkg-config openssl ] ++ lib.optionals stdenv.isDarwin [
            frameworks.SystemConfiguration
            frameworks.CoreServices
          ];
        };

        packages.default = pkgs.rustPlatform.buildRustPackage {
          name = "cch23-variasuit32"; # Same that is in Cargo.toml

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };
        };
      }
    );
}

