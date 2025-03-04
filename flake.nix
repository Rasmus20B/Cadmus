
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
          pkgs.docker
          pkgs.docker-compose
        ];

        shellHook = ''
          export DATABASE_URL=postgres://user:password@localhost/dbname
          echo "Starting postgres docker container..."
          docker-compose up -d
          echo "Development environment ready!"
        '';
        };
      }
    );
}
