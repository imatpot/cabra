{
  description = "A toolbox for language construction";

  inputs = {
    nixpkgs.url = "nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";

    treefmt-nix.url = "github:numtide/treefmt-nix";

    rust = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs:
    inputs.utils.lib.eachDefaultSystem (
      system: let
        pkgs = inputs.nixpkgs.legacyPackages.${system}.appendOverlays [
          inputs.rust.overlays.default
        ];
        rust = rec {
          toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          platform = pkgs.makeRustPlatform {
            cargo = toolchain;
            rustc = toolchain;
          };
        };
        waylandPkgs = with pkgs;
          lib.optionals stdenv.isLinux [
            wayland
            libxkbcommon
            mesa
            libGL
          ];
        treefmt = inputs.treefmt-nix.lib.evalModule pkgs {
          projectRootFile = "flake.nix";
          programs = {
            alejandra.enable = true;
            rustfmt.enable = true;
          };
        };
      in {
        packages = let
          manifest = pkgs.lib.importTOML ./Cargo.toml;
        in rec {
          default = cabra;

          cabra = rust.platform.buildRustPackage {
            inherit (manifest.package) name version;

            cargoLock.lockFile = ./Cargo.lock;
            src = pkgs.lib.cleanSource ./.;

            nativeBuildInputs = with pkgs; [
              pkg-config
            ];
          };
        };

        devShells.default = pkgs.mkShell {
          name = "cabra";

          buildInputs = with pkgs;
            [
              rust.toolchain
              pkg-config
            ]
            ++ waylandPkgs;

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath waylandPkgs;
        };

        formatter = treefmt.config.build.wrapper;
      }
    );
}
