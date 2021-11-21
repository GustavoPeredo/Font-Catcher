{
  name = "font-catcher";
  description = "A command line font package manager";
  inputs.nixpkgs.follows = "nix/nixpkgs";

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let 
         pkgs = import nixpkgs { overlays = [
	   (self: super: {
      	     neovim = super.neovim.override {
      	       viAlias = true;
               vimAlias = true;
               configure = {
                 packages.myVimPackage = with pkgs.vimPlugins; {
                   start = [ vim-nix coc-nvim coc-rls];
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
           })
	 ]};
	cargoToml = (builtins.fromTOML (builtins.readFile ./Cargo.toml));
	supportedSystems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" ];
        forAllSystems = f: nixpkgs.lib.genAttrs supportedSystems (system: f system);
      in {
        overlay = final: prev: {
          "${cargoToml.package.name}" = 
	    let 
	      pkgs = import nixpkgs { };
	    in
	      pkgs.stdenv.mkDerivation {
	        pname = "${cargoToml.package.name}";
		version = "${cargoToml.package.version}"
		src = ./.
		nativeBuildInputs = with pkgs; [
		  pkgs.rustc
		  pkgs.cargo
	    	  pkgs.pkg-config
		  pkgs.freetype
            	  pkgs.openssl
            	  pkgs.cmake 
            	  pkgs.llvm
            	  pkgs.gnumake
            	  pkgs.expat
            	  pkgs.fontconfig
                ];
		buildPhase = ''
                  cargo build --release
  		'';
		installPhase = ''
                  install -Dm775 ./target/release/${cargoToml.package.name} $out/bin/${cargoToml.package.name}
                '';
	      };
	};

	packages = forAllSystems (system:
          let
            pkgs = import nixpkgs {
              inherit system;
              overlays = [
                self.overlay
              ];
            };
          in
            {
              "${cargoToml.package.name}" = pkgs."${cargoToml.package.name}";
        });

	defaultPackage = forAllSystems (system: (import nixpkgs {
          inherit system;
          overlays = [ self.overlay ];
        })."${cargoToml.package.name}");

        devShell = pkgs.mkShell {
          nativeBuildInputs = [ 
            pkgs.rustup
	    pkgs.rust-analyzer
      	    
	    pkgs.neovim
          ];
	};
      }
   );
}
