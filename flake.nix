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
            devShells = {
              # Default development shell
              default = config.rust-project.crane-lib.devShell {
                inputsFrom = [
                  config.pre-commit.devShell
                  config.treefmt.build.devShell
                ];
              };

              # Benchmark shell
              benchmark =
                  let
                      benchmarkPkgs = import (builtins.fetchTarball {
                        url = "https://github.com/NixOS/nixpkgs/archive/nixos-24.05.tar.gz";
                        sha256 = "0zydsqiaz8qi4zd63zsb2gij2p614cgkcaisnk11wjy3nmiq0x1s";
                      }) {
                        inherit (pkgs) system;
                        config = {};
                      };

                      # Build scantree from source (dependency of dirhash)
                      scantree = benchmarkPkgs.python3Packages.buildPythonPackage rec {
                        pname = "scantree";
                        version = "0.0.4";

                        src = benchmarkPkgs.fetchPypi {
                          inherit pname version;
                          sha256 = "Fb1cskSDsE2yxwZTYE6Oo1IumAh9t+OKuEgvBTmEwKw=";
                        };

                        # Add versioneer as build dependency
                        nativeBuildInputs = with benchmarkPkgs.python3Packages; [
                          versioneer
                        ];

                        # No dependencies
                        propagatedBuildInputs = with benchmarkPkgs.python3Packages; [
                          attrs
                          pathspec
                        ];

                        # Disable tests
                        doCheck = false;

                        meta = {
                          description = "Recursive directory iterator supporting exclusion of files and directories";
                          homepage = "https://github.com/andhus/scantree";
                          license = benchmarkPkgs.lib.licenses.mit;
                        };
                      };

                      # Build dirhash-python from source with pinned version
                      dirhash = benchmarkPkgs.python3Packages.buildPythonApplication rec {
                        pname = "dirhash";
                        version = "0.5.0";

                        src = benchmarkPkgs.fetchFromGitHub {
                          owner = "andhus";
                          repo = "dirhash-python";
                          rev = "v${version}";
                          sha256 = "lTg9GwHArwHnZC7UwQEGa9TVdLKbkVANV84hkKsqQUo=";
                        };

                        # Dependencies from setup.py
                        propagatedBuildInputs = with benchmarkPkgs.python3Packages; [
                          scantree  # install_requires=["scantree>=0.0.4"]
                        ];

                        # Build dependencies - versioneer and git for version detection
                        nativeBuildInputs = with benchmarkPkgs; [
                          git
                        ] ++ (with benchmarkPkgs.python3Packages; [
                          versioneer
                        ]);

                        # Disable tests during build (optional, speeds up build)
                        doCheck = false;

                        meta = {
                          description = "Python module and CLI for hashing of file system directories";
                          homepage = "https://github.com/andhus/dirhash-python";
                          license = benchmarkPkgs.lib.licenses.mit;
                        };
                      };
                    in
                    config.rust-project.crane-lib.devShell {
                      packages = with benchmarkPkgs; [
                        hyperfine
                        b3sum
                        coreutils
                        findutils
                        gnugrep
                        gnused
                        gawk
                        git
                        bash
                        dirhash
                      ] ++ [
                        self'.packages.paq
                      ];

                      shellHook = ''
                        echo "=== paq benchmark environment (pinned versions) ==="
                        echo "nixos 24.05"
                        echo "$(paq --version)"
                        echo ""
                        echo "Package versions:"
                        echo "  - $(hyperfine --version)"
                        echo "  - $(b3sum --version)"
                        echo "  - dirhash ${dirhash.version}"
                        echo "  - $(sha256sum --version | head -n1)"
                        echo "  - $(git --version)"
                        echo ""
                        echo "Run: ./benches/hyperfine.sh"
                      '';
                  };
              };
          };
      }
    );
}