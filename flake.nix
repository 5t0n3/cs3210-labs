{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  inputs.crane.url = "github:ipetkov/crane";
  inputs.rust-overlay = {
    url = "github:oxalica/rust-overlay";
    inputs.nixpkgs.follows = "nixpkgs";
  };
  inputs.rustlings-git = {
    url = "github:rust-lang/rustlings/5.5.1";
    # don't use as a flake since they don't expose the rustlings package by itself :(
    flake = false;
  };
  inputs.cargo-binutils-git = {
    url = "github:rust-embedded/cargo-binutils";
    flake = false;
  };

  outputs = {
    self,
    nixpkgs,
    crane,
    rust-overlay,
    rustlings-git,
    cargo-binutils-git,
  }: let
    system = "x86_64-linux";
    pkgs = import nixpkgs {
      inherit system;
      overlays = [rust-overlay.overlays.default];
    };

    rust-nightly = pkgs.rust-bin.selectLatestNightlyWith (toolchain:
      toolchain.default.override {
        extensions = ["rust-src" "clippy" "llvm-tools-preview" "rust-analyzer"];
        targets = ["aarch64-unknown-none"];
      });

    craneLib = (crane.mkLib pkgs).overrideToolchain rust-nightly;

    # borrowed from the main rustlings repo
    rustlings = craneLib.buildPackage {
      pname = "rustlings";
      version = "5.5.1";

      src = with pkgs.lib;
        cleanSourceWith {
          src = rustlings-git;
          # a function that returns a bool determining if the path should be included in the cleaned source
          filter = path: type: let
            # filename
            baseName = builtins.baseNameOf (toString path);
            # path from root directory
            path' = builtins.replaceStrings ["${rustlings-git}/"] [""] path;
            # checks if path is in the directory
            inDirectory = directory: hasPrefix directory path';
          in
            inDirectory "src"
            || inDirectory "tests"
            || hasPrefix "Cargo" baseName
            || baseName == "info.toml";
        };

      doCheck = false;
    };

    cargo-binutils = craneLib.buildPackage {
      pname = "cargo-binutils";
      version = "master";

      src = craneLib.cleanCargoSource cargo-binutils-git;
      cargoVendorDir = craneLib.vendorCargoDeps {cargoLock = ./nix/binutils-Cargo.lock;};
    };

    # this is hacky because of the dependencies on shim/xmodem :')
    ttywrite = let
      inherit (nixpkgs) lib;
      libPath = toString ./lib;
      ttywrite-and-deps = lib.cleanSourceWith {
        src = libPath;
        filter = path: _: (lib.hasPrefix "${libPath}/xmodem" path) || (lib.hasPrefix "${libPath}/ttywrite" path) || (lib.hasPrefix "${libPath}/shim" path);
      };
      src = craneLib.cleanCargoSource ttywrite-and-deps;
    in
      craneLib.buildPackage {
        pname = "ttywrite";
        version = "0.1.0";

        inherit src;

        sourceRoot = "source/ttywrite";
        cargoVendorDir = craneLib.vendorCargoDeps {cargoLock = "${libPath}/ttywrite/Cargo.lock";};
      };

    inherit (pkgs) pwndbg clang clang-tools qemu socat python3;
  in {
    # TODO: figure out what else to install
    devShells.${system}.default = pkgs.mkShell {
      LLVM_TOOLS_PATH = "${rust-nightly}/lib/rustlib/x86_64-unknown-linux-gnu/bin";
      LLVM_LD_PATH = "${rust-nightly}/lib/rustlib/x86_64-unknown-linux-gnu/bin/gcc-ld";
      HOST_TARGET = "x86_64-unknown-linux-gnu";

      packages = [
        # general stuff
        rust-nightly
        pwndbg
        qemu
        cargo-binutils
        python3

        # lab0
        rustlings

        # lab1
        clang
        clang-tools

        # lab2
        socat
        # ttywrite
      ];
    };

    formatter.${system} = pkgs.alejandra;
  };
}
