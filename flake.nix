{
  description = "Rust Template";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/07af2a322744d7a791f6e7424fc6e81eb6877a95";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
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
          version = "15.0.0";

          src = pkgs.fetchzip {
            url = "https://crates.io/api/v1/crates/${pname}/${version}/download";
            hash = "sha256-AXpMMoVEJBMoLRQ06T0uVkbG+8vCkYuLESWBepihVo4=";
            extension = "tar";
          };

          cargoHash = "sha256-70eQ/n3oPBt3vGft5NNjTruvKJPvE7K3TUreVmG8flE=";

          buildInputs = [
            (lib.optional stdenv.isDarwin frameworks.SystemConfiguration)
          ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            rust-bin.stable.latest.default
            rust-analyzer
            cargo-shuttle
            cargo-watch
            cch23-validator
          ];

          buildInputs = [ ] ++ lib.optionals stdenv.isDarwin [
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

