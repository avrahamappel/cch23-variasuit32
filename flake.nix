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
          version = "22.0.0";

          src = pkgs.fetchzip {
            url = "https://crates.io/api/v1/crates/${pname}/${version}/download";
            hash = "sha256-sVhPab3El+Il3BiCs6hR9uMy6xi7JpRvXHPP7Ta0PB4=";
            extension = "tar";
          };

          cargoHash = "sha256-9np7DwLpf6gNjrRG4l5YtWEZOXidPpuZ22KGcR08iV4=";

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

