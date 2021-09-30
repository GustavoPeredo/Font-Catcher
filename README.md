# Font Catcher

*A command line font package manager.*

## Installation

### Dependencies

Font Catcher relies on `curl` to download fonts, make sure to install it before using Font Catcher.

### Recommended

Download one of the following:

![Standard repo](https://github.com/GustavoPeredo/Font-Catcher/releases/download/v1.0.1/font-catcher.zip)

![Standard repo + Google Fonts (Recommended)](https://github.com/GustavoPeredo/Font-Catcher/releases/download/v1.0.1/font-catcher.g.zip)

Extract and copy to `/usr/bin`! That's it!

### Using Cargo

To install using the standard repo, run:

```
cargo install font-catcher
```

To install with Google Fonts:

1. Grab an API Key on ![Google Font's Website](https://developers.google.com/fonts/docs/developer_api)

2. Run:

```
GOOGLE_FONTS_KEY="YOUR API KEY HERE" cargo install font-catcher --features google_repo
```

# Usage

Font Catcher's commands are aimed at being easy and intuitive to use. If you have used `apt` or `dnf` as package managers once, you will notice the similarities. By default, Font Catcher comes with one repository which is currently WIP: ![Open Font Repository](https://github.com/GustavoPeredo/open-font-repository), but it is possible to use Google Fonts as a repository as well (and other repositories).

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

# To-improve!

* (D) Develop an update system for fonts
* (A) Show installed fonts as `[installed]` or `[system installed]`
* \(C\) Make `Open Font Repository` optional at compile time, but still default.
* (A) Allow to filter fonts by subsets, categories and lastModified
* \(C\) Add translations
* (B) Package to distributions
