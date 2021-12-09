{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = {self, flake-utils, nixpkgs }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        tomlFile = builtins.readFile ./Cargo.toml;
	      toml = builtins.fromTOML tomlFile;
      in
        {
          devShell = import ./shell.nix { inherit pkgs; };
          defaultPackage = (pkgs.makeRustPlatform {
            inherit (pkgs)
              rustc
              cargo;
          }).buildRustPackage {
            pname = toml.package.name;
            version = toml.package.version;

            PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig:${pkgs.freetype.dev}/lib/pkgconfig:${pkgs.expat.dev}/lib/pkgconfig";
            nativeBuildInputs = with pkgs; [
              pkg-config
              freetype
              openssl
              cmake
              llvm
              gnumake
              expat
              fontconfig
            ] ++ pkgs.lib.optionals stdenv.isDarwin [
              darwin.apple_sdk.frameworks.CoreText
              libiconv
            ];

            src = ./.;
            cargoSha256 = "sha256-Cuk8pep4FgXV9YxUyzYliJNUfrz9ok/z45FVYhikLYM";  
          };
          
    });
}
