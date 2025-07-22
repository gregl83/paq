{
  description = "Hash file or directory recursively.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
    flake-parts.url = "github:hercules-ci/flake-parts";
    nix-systems = {
      url = "github:nix-systems/default";
      flake = false;
    };
    rust-flake.url = "github:juspay/rust-flake";
  };

  outputs =
    inputs@{ flake-parts, nix-systems, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } (
      top@{
        config,
        withSystem,
        moduleWithSystem,
        ...
      }:
      {
        imports = [
          inputs.rust-flake.flakeModules.default
          inputs.rust-flake.flakeModules.nixpkgs
        ];
        systems = (import nix-systems);
        perSystem =
          {
            config,
            self',
            pkgs,
            ...
          }:
          {
            rust-project = {
              src = pkgs.lib.cleanSourceWith {
                src = config.rust-project.crane-lib.path ./.;
                filter = pkgs.lib.cleanSourceFilter;
              };
            };
            packages = {
              default = self'.packages.paq;
            };
            devShells.default = config.rust-project.crane-lib.devShell { };
          };
      }
    );
}
