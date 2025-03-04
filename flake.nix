
{
  description = "Development environment for Cadmus";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:/numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = [
          pkgs.rustc
          pkgs.cargo
          pkgs.proto
          pkgs.sqlx-cli
          pkgs.openssl
          pkgs.postgresql
        ];

        shellHook = ''
          export DATABASE_URL=postrges://user:password@localhost/dbname
          alias v="nvim" #bruh
          echo "Development environment ready!"
        '';
        };
      }
    );
}
