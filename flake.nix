{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  inputs.rust-overlay = {
    url = "github:oxalica/rust-overlay";
    inputs.nixpkgs.follows = "nixpkgs";
  };
  inputs.rustlings-git = {
    url = "github:rust-lang/rustlings/5.5.1";
    # don't use as a flake since they don't expose the rustlings package by itself :(
    flake = false;
  };

  outputs = {self, nixpkgs, rust-overlay, rustlings-git}:
    let system = "x86_64-linux";
        pkgs = import nixpkgs {
          inherit system;
          overlays = [rust-overlay.overlays.default];
        };

        # borrowed from the main rustlings repo
        rustlings = pkgs.rustPlatform.buildRustPackage {
          name = "rustlings";
            version = "5.5.1";

            src = with pkgs.lib; cleanSourceWith {
              src = rustlings-git;
              # a function that returns a bool determining if the path should be included in the cleaned source
              filter = path: type:
                let
                  # filename
                  baseName = builtins.baseNameOf (toString path);
                  # path from root directory
                  path' = builtins.replaceStrings [ "${rustlings-git}/" ] [ "" ] path;
                  # checks if path is in the directory
                  inDirectory = directory: hasPrefix directory path';
                in
                inDirectory "src" ||
                inDirectory "tests" ||
                hasPrefix "Cargo" baseName ||
                baseName == "info.toml";
            };

            doCheck = false;

            cargoLock.lockFile = "${rustlings-git}/Cargo.lock";
        };
        rust-nightly = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
          extensions = ["rust-src" "clippy" "llvm-tools-preview" "rust-analyzer"];
          targets = [ "aarch64-unknown-none" ];
        });
        inherit (pkgs) pwndbg qemu;
    in {
      # TODO: figure out what else to install
      devShells.${system}.default = pkgs.mkShell {
        packages = [rustlings rust-nightly pwndbg qemu];
      };

      formatter.${system} = pkgs.alejandra;
    };
}
