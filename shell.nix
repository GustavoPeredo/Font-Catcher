{ pkgs ? import <nixpkgs> {} }:
  pkgs.mkShell {
    nativeBuildInputs = [ 
      pkgs.rustup
      pkgs.pkg-config
      pkgs.freetype
      pkgs.openssl
      pkgs.cmake 
      pkgs.llvm
      pkgs.gnumake
      pkgs.expat
      pkgs.fontconfig
    ];
}
