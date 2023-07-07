{
  description = "An anyrun plugin that fetches crypto prices";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    nixpkgs,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = nixpkgs.legacyPackages.${system};
      in rec {
        devShell = with pkgs;
          mkShell {
            buildInputs = [
              cargo
              clippy
              rustc
              git
              rustfmt
              sqlite
              diesel-cli
            ];
          };

        packages = rec {
          anyrun-ha-assist = pkgs.callPackage ./nix {};
          default = anyrun-ha-assist;
        };

        anyrunPlugins = rec {
          anyrun-crypto= "${packages.default}/lib/libcryptorun.so";
          default = anyrun-crypto;
        };

        legacyPackages = packages;
      }
    );
}