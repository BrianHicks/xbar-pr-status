{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.inputs.nixpkgs.follows = "nixpkgs";
    naersk.url = "github:nmattia/naersk";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = { self, nixpkgs, flake-utils, naersk }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages."${system}";
        naersk-lib = naersk.lib."${system}";
        darwinInputs = if pkgs.stdenv.isDarwin then [ pkgs.xcbuild ] else [ ];
      in rec {
        # `nix build`
        packages.xbar-pr-status = naersk-lib.buildPackage {
          root = ./.;
          buildInputs = [ pkgs.libiconv pkgs.rustPackages.clippy ]
            ++ darwinInputs;

          doCheck = true;
          checkPhase = ''
            cargo test
            cargo clippy -- --deny warnings
          '';
        };
        defaultPackage = packages.xbar-pr-status;
        overlay = final: prev: { xbar-pr-status = packages.xbar-pr-status; };

        packages.update-github-graphql-schema =
          pkgs.writeShellScriptBin "update-github-graphql-schema" ''
            set -euo pipefail
            ROOT="$(${pkgs.git}/bin/git rev-parse --show-toplevel)"
            ${pkgs.curl}/bin/curl https://raw.githubusercontent.com/octokit/graphql-schema/master/schema.graphql > "$ROOT/src/github.schema.graphql"
          '';

        # `nix develop`
        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs;
            [
              cargo
              cargo-edit
              cargo-insta
              rustPackages.clippy
              rustc
              rustfmt

              # for some reason this seems to be required, especially on macOS
              libiconv

              # scripts
              packages.update-github-graphql-schema
            ] ++ darwinInputs;
        };
      });
}
