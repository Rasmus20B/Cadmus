
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
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = [
          pkgs.rustc
          pkgs.cargo
          pkgs.proto
          pkgs.sqlx-cli
          pkgs.openssl
          pkgs.postgresql
          pkgs.docker
          pkgs.docker-compose
          pkgs.openssl

          pkgs.nodejs
        ];

        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [ pkgs.openssl ];

        shellHook = ''
          export DATABASE_URL=postgres://user:password@localhost/cadmus_db
          echo "Development environment ready!"
        '';
        };
      }
    );
}
