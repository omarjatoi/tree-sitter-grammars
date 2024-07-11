{
  description = ''
    Repository containing tree-sitter grammars for many different languages available
    in one place, and a small CLI utility to add and update grammars.
  '';

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/master";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        inherit (pkgs.darwin.apple_sdk.frameworks) Security;
        inherit (pkgs.lib) optionals;
        inherit (pkgs.stdenv) isDarwin;

        rustVersion = pkgs.rust-bin.stable.latest.default;

        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustVersion;
          rustc = rustVersion;
        };

      in {
        defaultPackage = with pkgs;
          rustPlatform.buildRustPackage {
            pname = "tree-sitter-grammars";
            version = "0.1.0";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;

            buildInputs = [ openssl ] ++ optionals isDarwin [ Security ];
            nativeBuildInputs = [ pkg-config libiconv ];

            PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
          };

        devShell = with pkgs;
          mkShell {
            nativeBuildInputs =
              [ clippy libiconv pkg-config rust-analyzer rustVersion rustfmt ];
            buildInputs = [ openssl ] ++ optionals isDarwin [ Security ];
            shellHook = ''
              echo "Entering dev shell for 'tree-sitter-grammars' project."
            '';
            PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
          };
      });
}
