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
          defaultPackage = (pkgs.makeRustPlatform {
            inherit (pkgs)
              rustc
              cargo;
          }).buildRustPackage {
            pname = toml.package.name;
            version = toml.package.version;

            PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig:${pkgs.freetype.dev}/lib/pkgconfig:${pkgs.expat.dev}/lib/pkgconfig:${pkgs.curl.dev}/lib/pkgconfig";
	    nativeBuildInputs = with pkgs; [
	      pkg-config
	    ];

            buildInputs = with pkgs; [
              freetype
              openssl
              cmake
              llvm
              gnumake
              expat
              fontconfig
              curl
            ] ++ pkgs.lib.optionals stdenv.isDarwin [
              darwin.apple_sdk.frameworks.CoreText
              libiconv
            ];

            src = ./.;
            cargoSha256 = "lJG+nj9i1bxjjN+/tQfsRZ2HYsFzmRheEJyEzrXiOZM=";
          };
          
    });
}
