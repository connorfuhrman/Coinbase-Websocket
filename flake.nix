{
  description = "A Rust Websocket client package";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    
    flake-utils.url = "github:numtide/flake-utils";
    
    # Add crane for better Rust packaging
    crane = {
      url = "github:ipetkov/crane";
      # inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, crane, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        
        # Use stable rust
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };

        # Initialize crane with our toolchain
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        # Create our package
        rust-websocket-client = craneLib.buildPackage {
          src = craneLib.cleanCargoSource (craneLib.path ./.);
          
          buildInputs = with pkgs; [
            # Runtime dependencies
            openssl
          ];
          
          nativeBuildInputs = with pkgs; [
            # Build-time dependencies
            pkg-config
            openssl.dev
          ];
        };
      in
      {
        # Export the package
        packages.default = rust-websocket-client;
        
        # Keep the development shell
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustToolchain
            pkg-config
            openssl
            openssl.dev
          ];

          shellHook = ''
            export PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig"
          '';
        };
      }
    );
}
