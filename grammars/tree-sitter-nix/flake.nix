{
  description = "tree-sitter-nix";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    flake-utils.url = "github:numtide/flake-utils";

    npmlock2nix = {
      url = "github:nix-community/npmlock2nix";
      flake = false;
    };

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    nix-github-actions.url = "github:nix-community/nix-github-actions";
    nix-github-actions.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, flake-utils, npmlock2nix, crane, nix-github-actions }: (
    (flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        inherit (pkgs) lib;

        npmlock2nix' = pkgs.callPackage npmlock2nix { };
        craneLib = crane.lib.${system};

      in
      {
        checks =
          let
            # shellPackages = (pkgs.callPackage ./shell.nix { }).packages;

            # If the generated code differs from the checked in we need
            # to check in the newly generated sources.
            mkCheck = name: check: pkgs.runCommand name
              {
                inherit (self.devShells.${system}.default) nativeBuildInputs;
              } ''
              cp -rv ${self} src
              chmod +w -R src
              cd src

              ${check}

              touch $out
            '';

          in
          {
            build = self.packages.${system}.tree-sitter-nix;

            editorconfig = mkCheck "editorconfig" "editorconfig-checker";

            # If the generated code differs from the checked in we need
            # to check in the newly generated sources.
            generated-diff = mkCheck "generated-diff" ''
              HOME=. npm run generate
              diff -r src/ ${self}/src
            '';

            treefmt = mkCheck "treefmt" "treefmt --no-cache --fail-on-change";

            rust-bindings = craneLib.buildPackage {
              src = self;
            };

          } // lib.optionalAttrs (!pkgs.stdenv.isDarwin) {
            # Requires xcode
            node-bindings = npmlock2nix'.v2.build {
              src = self;
              inherit (self.devShells.${system}.default) nativeBuildInputs;
              inherit (pkgs) nodejs;

              buildCommands = [
                "${pkgs.nodePackages.node-gyp}/bin/node-gyp configure"
                "npm run build"
              ];

              installPhase = ''
                touch $out
              '';
            };

          };

        packages.tree-sitter-nix = pkgs.callPackage ./default.nix { src = self; };
        packages.default = self.packages.${system}.tree-sitter-nix;
        devShells.default = pkgs.callPackage ./shell.nix { };
      })) // {

      githubActions = nix-github-actions.lib.mkGithubMatrix {
        # Inherit GHA actions matrix from a subset of platforms supported by hosted runners
        checks = {
          inherit (self.checks) x86_64-linux;

          # Don't run linters on darwin as it's just scheduling overhead
          x86_64-darwin = builtins.removeAttrs self.checks.x86_64-darwin [ "editorconfig" "generated-diff" "treefmt" ];
        };
      };

    }
  );
}
