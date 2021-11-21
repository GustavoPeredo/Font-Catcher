{ pkgs ? import <nixpkgs> {
  overlays = [ (self: super: {
    neovim = super.neovim.override {
      viAlias = true;
      vimAlias = true;
      configure = {
        packages.myVimPackage = with pkgs.vimPlugins; {
          start = [ vim-nix coc-nvim coc-rls ];
        };
	customRC = ''
	  set ignorecase
	  set mouse=v
	  set hlsearch
	  set number
	  set cc=80
	  filetype plugin indent on
	  set ttyfast
          highlight ColorColumn ctermbg=7
        '';
      };
    };
  })];
} }:
  pkgs.mkShell {
    nativeBuildInputs = with pkgs; [ 
      rustc
      rust-analyzer
      rustfmt
      rls
      cargo

      pkg-config
      freetype
      openssl
      cmake 
      llvm
      gnumake
      expat
      fontconfig

      neovim
      nodejs
      rustup
    ];
}
