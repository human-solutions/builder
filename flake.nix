{
  description = "Command line tool for building web assets, wasm and mobile libraries";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
  };

  outputs = { self, nixpkgs }: 
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

      defaultPackage = build packages;

      # FUNCTIONS

      url = { triple, version }: "https://github.com/human-solutions/builder/releases/download/v${version}/builder-${triple}.tar.xz";

      build = package: builtins.listToAttrs (map (system: {
        name = system;
        value = with import nixpkgs { system = package.${system}.label; };
          stdenvNoCC.mkDerivation rec {
            inherit name version;

            # https://nixos.wiki/wiki/Packaging/Binaries
            src = pkgs.fetchurl {
              url = url { 
                triple = package.${system}.triple;
                version = version; 
              };
              # Get the cheksum from the release on github
              # Convert it to base64
              # Then prefix it with 'sha256-'
              sha256 = package.${system}.checksum;
            };

            sourceRoot = ".";

            installPhase = ''
            install -m755 -D builder-${package.${system}.triple}/builder $out/bin/builder
            '';

            meta = with lib; {
              homepage = "https://github.com/human-solutions/builder";
              description = "Command line tool for building web assets, wasm and mobile libraries";
              platforms = platforms.${package.${system}.platform};
            };
          };
      }) (builtins.attrNames package));

    in
    {
      inherit defaultPackage;

      devShells.default = with nixpkgs; mkShell {
        buildInputs = [
          cargo
          rustc
        ];
      };
    };

}