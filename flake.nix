{
  description = "The devShell";

  inputs = {
    nixpkgs.url      = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url  = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk/master";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, naersk }:
    let
      name = "builder";
      version = "0.0.4";
      packages = {
        aarch64-darwin = {
          label = "aarch64-darwin";
          triple = "aarch64-apple-darwin";
          checksum = "sha256-CVP8uIFVV1ZakiClpqI79gP0nJTqTZo3QXq9Z2gYGjA=";
          platform = "darwin";
        };
        x86_64-darwin = {
          label = "x86_64-darwin";
          triple = "x86_64-apple-darwin";
          checksum = "sha256-KMprfKliocjjVQKnpq1GwadSsCI8bSVIqGN+F+KJ1qs=";
          platform = "darwin";
        };
        x86_64-linux = {
          label = "x86_64-linux";
          triple = "x86_64-unknown-linux-gnu";
          checksum = "sha256-Tnka+d8svkZrlX0q7NazgHowMoVftp2ZIsf37j5MEgg=";
          platform = "linux";
        };
      };

      # -- FUNCTIONS --

      url = { triple, version }: "https://github.com/human-solutions/builder/releases/download/v${version}/builder-${triple}.tar.xz";
    in
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        naersk-lib = pkgs.callPackage naersk { };
        buildInputs = [
          pkgs.dart-sass
          pkgs.fontforge
        ];
      in
      {
        defaultPackage = if builtins.hasAttr system packages then
          with import nixpkgs { system = packages.${system}.label; };
            stdenvNoCC.mkDerivation rec {
              inherit name version buildInputs;

              # https://nixos.wiki/wiki/Packaging/Binaries
              src = pkgs.fetchurl {
                url = url { 
                  triple = packages.${system}.triple;
                  version = version; 
                };
                # Get the cheksum from the release on github
                # Convert it to base64
                # Then prefix it with 'sha256-'
                sha256 = packages.${system}.checksum;
              };

              sourceRoot = ".";

              installPhase = ''
              install -m755 -D builder-${packages.${system}.triple}/builder $out/bin/builder
              '';

              meta = with lib; {
                homepage = "https://github.com/human-solutions/builder";
                description = "Command line tool for building web assets, wasm and mobile libraries";
                platforms = platforms.${packages.${system}.platform};
              };
            }
        else
          naersk-lib.buildPackage {
            inherit name version buildInputs;
            src = ./.;
          };
        devShells.default = with pkgs; mkShell {
          buildInputs = [
            dart-sass
            fontforge
            (rust-bin.stable.latest.default.override {
              targets = [ "wasm32-unknown-unknown" ];
            })
          ];
        };
      }
    );
}
