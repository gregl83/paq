{
  description = "Hash file or directory recursively.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
    flake-parts.url = "github:hercules-ci/flake-parts";
    git-hooks-nix = {
      url = "github:cachix/git-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nix-systems = {
      url = "github:nix-systems/default";
      flake = false;
    };
    rust-flake.url = "github:juspay/rust-flake";
    treefmt-nix.url = "github:numtide/treefmt-nix";
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
          inputs.git-hooks-nix.flakeModule
          inputs.rust-flake.flakeModules.default
          inputs.rust-flake.flakeModules.nixpkgs
          inputs.treefmt-nix.flakeModule
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
            pre-commit = {
              check.enable = true;
              settings.hooks = {
                editorconfig-checker.enable = true;
                ripsecrets.enable = true;
                taplo.enable = true;
                treefmt.enable = true;
                typos.enable = true;
              };
            };
            rust-project = {
              src = pkgs.lib.cleanSourceWith {
                src = config.rust-project.crane-lib.path ./.;
                filter = pkgs.lib.cleanSourceFilter;
              };
            };
            treefmt.config = {
              projectRootFile = ".git/config";
              flakeCheck = false; # use pre-commit's check instead
              programs = {
                nixfmt.enable = true; # nix
                prettier.enable = true;
                shellcheck.enable = true;
                shfmt = {
                  enable = true;
                  indent_size = 2;
                };
                taplo.enable = true;
              };
            };
            packages = {
              default = self'.packages.paq;
            };
            devShells.default = config.rust-project.crane-lib.devShell {
              inputsFrom = [
                config.pre-commit.devShell
                config.treefmt.build.devShell
              ];
            };
          };
      }
    );
}
