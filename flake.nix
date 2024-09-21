# Enter the environment with `nix develop -c $YOUR_SHELL`
{
  description = "Nix dev environment";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    systems.url = "github:nix-systems/default";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    treefmt-nix.url = "github:numtide/treefmt-nix";
  };

  outputs =
    inputs@{
      flake-parts,
      nixpkgs,
      rust-overlay,
      treefmt-nix,
      ...
    }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = import inputs.systems;

      imports = [
        treefmt-nix.flakeModule
      ];

      perSystem =
        {
          config,
          self',
          pkgs,
          lib,
          system,
          ...
        }:
        {
          _module.args.pkgs = import nixpkgs {
            inherit system;
            overlays = [
              rust-overlay.overlays.default
            ];
          };

          treefmt.config = {
            projectRootFile = "flake.nix";
            programs = {
              nixfmt.enable = true;
              rustfmt.enable = true;
            };
          };

          # Cranelift codegen backend.
          #
          # Use `nix develop -c $YOUR_SHELL`
          devShells.cranelift =
            # HACK: There doesn't seem to be a way to pass default flags to
            # cargo without touching .cargo/config.toml.
            let
              cargo_alias = pkgs.writeShellScriptBin "car" ''
                cargo -Z codegen-backend $@
              '';
            in
            pkgs.mkShell (
              (import ./nixModules/devShell.nix {
                inherit config pkgs;
                libraries = [ ];
                rustToolchainFile = ./rust-toolchain-cranelift.toml;
                moreBuildInputs = [ cargo_alias ];
              })
              // {
                CARGO_PROFILE_DEV_CODEGEN_BACKEND = "cranelift";
              }
            );

          # Stable toolchain with LLVM codegen.
          #
          # Use `nix develop .#stable -c $YOUR_SHELL`
          devShells.default =
            let
              cargo_alias = pkgs.writeShellScriptBin "car" ''
                cargo $@
              '';
            in
            pkgs.mkShell (
              import ./nixModules/devShell.nix {
                inherit config pkgs;
                libraries = [ ];
                rustToolchainFile = ./rust-toolchain.toml;
                moreBuildInputs = [ cargo_alias ];
              }
            );
        };
    };
}
