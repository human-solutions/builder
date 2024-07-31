{
  description = "Command line tool for building web assets, wasm and mobile libraries";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
  };

  outputs = { self, nixpkgs }: 
    let
      name = "builder";
      version = "0.0.3";

      packages = {
        aarch64-darwin = {
          label = "aarch64-darwin";
          triple = "aarch64-apple-darwin";
          checksum = "sha256-o82EeaeyppnCawV5F4pJNAsUlr2TEHHnHmQDyH9Ii9k=";
          platform = "darwin";
        };
        x86_64-darwin = {
          label = "x86_64-darwin";
          triple = "x86_64-apple-darwin";
          checksum = "sha256-POPubK7eZw4UxFIaPingI9xPy6ImihAfrtA1BIdF4+s=";
          platform = "darwin";
        };
        x86_64-linux = {
          label = "x86_64-linux";
          triple = "x86_64-unknown-linux-gnu";
          checksum = "sha256-nIg7sedGu8+rHj20OSE0q5Sc2VgCb+ADfydBqUvDvsA=";
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
    };

}