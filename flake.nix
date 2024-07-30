{
  description = "Command line tool for building web assets, wasm and mobile libraries";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
  };

  outputs = {self, nixpkgs}: {
    defaultPackage.aarch64-darwin =
      with import nixpkgs { system = "aarch64-darwin"; };

      stdenvNoCC.mkDerivation rec {
        name = "builder";

        version = "0.0.3";

        # https://nixos.wiki/wiki/Packaging/Binaries
        src = pkgs.fetchurl {
          url = "https://github.com/human-solutions/builder/releases/download/v${version}/builder-aarch64-apple-darwin.tar.xz";
          sha256 = "sha256-o82EeaeyppnCawV5F4pJNAsUlr2TEHHnHmQDyH9Ii9k=";
        };

        sourceRoot = ".";

        installPhase = ''
        install -m755 -D builder-aarch64-apple-darwin/builder $out/bin/builder
        '';

        meta = with lib; {
          homepage = "https://github.com/human-solutions/builder";
          description = "Command line tool for building web assets, wasm and mobile libraries";
          platforms = platforms.darwin;
        };
      };

    defaultPackage.x86_64-darwin =
      with import nixpkgs { system = "x86_64-darwin"; };

      stdenvNoCC.mkDerivation rec {
        name = "builder";

        version = "0.0.3";

        # https://nixos.wiki/wiki/Packaging/Binaries
        src = pkgs.fetchurl {
          url = "https://github.com/human-solutions/builder/releases/download/v${version}/builder-x86_64-apple-darwin.tar.xz";
          sha256 = "sha256-o82EeaeyppnCawV5F4pJNAsUlr2TEHHnHmQDyH9Ii9k=";
        };

        sourceRoot = ".";

        installPhase = ''
        install -m755 -D builder-x86_64-apple-darwin/builder $out/bin/builder
        '';

        meta = with lib; {
          homepage = "https://github.com/human-solutions/builder";
          description = "Command line tool for building web assets, wasm and mobile libraries";
          platforms = platforms.darwin;
        };
      };

    defaultPackage.x86_64-linux =
      with import nixpkgs { system = "x86_64-linux"; };

      stdenvNoCC.mkDerivation rec {
        name = "builder";

        version = "0.0.3";

        # https://nixos.wiki/wiki/Packaging/Binaries
        src = pkgs.fetchurl {
          url = "https://github.com/human-solutions/builder/releases/download/v${version}/builder-x86_64-unknown-linux-gnu.tar.xz";
          sha256 = "sha256-o82EeaeyppnCawV5F4pJNAsUlr2TEHHnHmQDyH9Ii9k=";
        };

        sourceRoot = ".";

        installPhase = ''
        install -m755 -D builder-x86_64-unknown-linux-gnu/builder $out/bin/builder
        '';

        meta = with lib; {
          homepage = "https://github.com/human-solutions/builder";
          description = "Command line tool for building web assets, wasm and mobile libraries";
          platforms = platforms.linux;
        };
      };
  };
}