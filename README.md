# Font Catcher

*A cross-platform command-line utility and a high-level library for user and system font management*

Cool badges :P

![main](https://github.com/GustavoPeredo/Font-Catcher/actions/workflows/rust.yml/badge.svg)
![publishing](https://github.com/GustavoPeredo/Font-Catcher/actions/workflows/publish-crate.yml/badge.svg)
![crate downloads](https://shields.io/crates/d/font-catcher)
![built with nix](https://builtwithnix.org/badge.svg)

## User Quick Start

### Installation

Head to the [releases](https://github.com/GustavoPeredo/Font-Catcher/releases),
download your preferred version (latest versions are recommended),
copy the file to /usr/bin and that's it!

NOTE: There are two releases (common and with [Google Fonts](https://fonts.google.com))
for now it is recommended to download the Google Fonts version to have a broader
variety of fonts, but this might change in the future, see
[Open Font Repository](https://github.com/GustavoPeredo/open-font-repository)

### Basic Usage

Font Catcher's commands are aimed at being easy and intuitive to use. 
If you have used `apt` or `dnf` as package managers once,
you will notice the similarities. By default, Font Catcher comes with one
repository which is currently WIP: 
![Open Font Repository](https://github.com/GustavoPeredo/open-font-repository),
but it is possible to use Google Fonts as a repository as well
(and other repositories).

To search for a font:

```
font-catcher search font-name
```

To install a font:

```
font-catcher install font-name
```

To remove a font:

```
font-catcher remove font-name
```

That's it! (For the most part)

## Developer Quick Start

### Adding to your project

`font-catcher` makes use of some low-level libraries that may not be installed
in your system.
```
sudo apt install pkg-config libfreetype-dev openssl librust-openssl-dev cmake llvm make expat fontconfig fontconfig1-dev
```

After installing these, add to your cargo file:
```
[dependencies]
font-catcher = "2.0.0"
```

### Using the library 

Two things are nescessary to make use of the library: Import it and initialize
it.

```rust
use font-catcher as font_catcher;

let main() {
    let fonts_hashmap = font_catcher::init()?;
}
```

This returns a 
[HashMap](https://doc.rust-lang.org/std/collections/struct.HashMap.html)
where the keys are font names and the value is a struct with plenty of
useful functions, in the following example, we will be installing and
removing the [Agave font](https://github.com/blobject/agave).

```rust
use font-catcher as font_catcher;

let main() {
    let fonts_hashmap = font_catcher::init()?;

    match fonts_hashmap.get("Agave") {
    	// Checks if the font exists
        Some(font) => {font.install_to_user(None, true)?;},
	// None -> This means the font will be downloaded from any repo
	// available.
	// true -> Gives terminal output of the operation.
	None => {println!("No Agave font found!");}
	// Prints a message if the font is not to be found
    }

    match fonts_hashmap.get("Agave") {
    	// Checks if the font exists
        Some(font) => {font.uninstall_from_user(true)?;},
	// true -> Gives terminal output of the operation.
	None => {println!("No Agave font found!");}
	// Prints a message if the font is not to be found
    }
}
```

### Docs

More examples can be found on the [main.rs file](https://github.com/GustavoPeredo/Font-Catcher/blob/main/src/main.rs) for the time being.

## Contributor Quick Start

### Setup with Nix

The first step is to clone the git repository:

```
git clone https://github.com/GustavoPeredo/Font-Catcher.git
cd Font-Catcher
```

Then, [nix](https://nixos.org/) makes it very easy to start
working on this project. Simply download [nix](https://nixos.org/download.html)
on any distribution or MacOS and:

```
nix-shell
```

*BAM!* All dependencies, rust and neovim with plugins will be installed.

### Compiling

For common hacking
```
cargo build
```
should be enough, but if you want to compile with Google Fonts repository,
then you will have to grab a key from 
![Google Font's Website](https://developers.google.com/fonts/docs/developer_api)
, then compile enabling the `google_repo` feature:

```
GOOGLE_FONTS_KEY="YOUR API KEY HERE" cargo build --features google_repo
```

### Versioning

v1.0.0 -> v2.0.0
Major releases change the api completely

v1.0.0 -> v1.1.0
Middle releases add features and may deprecate functions/function outputs

v1.0.0 -> v1.1.1
Minor releases contian bug fixes and misc updates


## Other documentation

## Further Usage

You can download fonts to a specific directory instead of installing them directly:

```
font-catcher download /desired/path Agave
```

You can specify which repository to search, install and download fonts from by passing the `--repo` flag before the fonts:

Example:

```
font-catcher install --repo "Google Fonts" Roboto

font-catcher search --repo "Open Font Repository" Aga

font-catcher download ~/Downloads --repo "Open Font Repository" Agave
``` 

It's possible to install, download and remove multiple fonts at once:

```
font-catcher install font1 font2 font3

font-catcher remove font1 font2 font3
```

To update the font catalogs to the latest versions, run:

```
font-catcher update-repos
```

## Adding repositories

### Editing the `repos.conf` file

The simples way to add another repo is by editing the `repos.conf` file, located under your data file inside a font-catcher folder (normally `~/.local/share/font-catcher`). If the file doesn't exist, create a new one.

This is a template for a repository:

```
[[repo]]
name = "Open Font Repository Local"
url = "https://raw.githubusercontent.com/GustavoPeredo/open-font-repository/main/fonts.json"

```

If your repository has an API key, add `{API_KEY}` where the API key should be placed in the url, example:

```
[[repo]]
name = "Google Fonts Local"
url = "https://www.googleapis.com/webfonts/v1/webfonts?key={API_KEY}"
key = "KEY"

```

You can add as many repositories as you want, just append them to the file like so:

```
[[repo]]
name = "Open Font Repository Local"
url = "https://raw.githubusercontent.com/GustavoPeredo/open-font-repository/main/fonts.json"

[[repo]]
name = "Google Fonts Local"
url = "https://www.googleapis.com/webfonts/v1/webfonts?key={API_KEY}"
key = "KEY"
```

### Adding to the source code

To add a repository to the source code, try following the example present in `src/repo.rs`:

```
Repository {
                name: "Open Font Repository".to_string(),
                url: "https://raw.githubusercontent.com/GustavoPeredo/open-font-repository/main/fonts.json".to_string(),
                key: None,
            }
```

Maybe there are other font repositories compatible with this software that I'm unaware of, it would be nice to have them as options at compilation time!
