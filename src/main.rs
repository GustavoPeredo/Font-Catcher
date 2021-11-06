use std::collections::HashMap;
use std::env::args;
use std::fs::{self, create_dir_all, File};
use std::io::Write;

#[cfg(target_os = "windows")]
use std::os::windows::fs::MetadataExt;

#[cfg(target_os = "linux")]
use std::os::linux::fs::MetadataExt;

use std::path::PathBuf;
use std::str;
use std::time;

use dirs::data_dir;

use font_kit::handle::Handle;
use font_kit::source::SystemSource;

use chrono::DateTime;
use chrono::offset::Utc;

use curl::easy::Easy;

use serde_json;
use toml;

mod repo;

#[derive(Debug)]
struct FontFile {
    variant: String,
    path: PathBuf,
    system: bool,
    creation_date: time::SystemTime,
}

fn get_share_dir() -> PathBuf {
    let mut share = match data_dir() {
        Some(p) => p,
        _ => {
            println!("Couldn't solve for dir, using current");
            PathBuf::from("./")
        }
    };
    share.push("font-catcher");
    create_dir_all(&share).expect("Couldn't create dir!");
    share
}

fn get_local_repos(repo_file: &PathBuf) -> Vec<repo::Repository> {
    let repositories: repo::Repositories = toml::from_str(
        &fs::read_to_string(repo_file)
            .expect("Unable to read repositories file")
    ).expect("Failed to read repositories file");
    repositories.repo
}

fn get_repo_json(url: &str) -> String {
    if cfg!(target_os = "windows") {
        return "".to_string();
    }
    match str::from_utf8(&download(url)) {
        Ok(v) => v.to_string(),
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    }
}

fn get_repo_as_file(name: &str) -> String {
    format!("{}{}", name, ".json")
}

fn update_repos(repos: &Vec<repo::Repository>, repos_dir: &PathBuf) {
    create_dir_all(repos_dir).expect("Couldn't create direcotires!");
    for repo in repos.iter() {
        println!("Updating {}...", &repo.name);
        fs::write(
            repos_dir.join(get_repo_as_file(&repo.name)),
            match &repo.key {
                Some(key) => get_repo_json(&repo.url.replace("{API_KEY}", &key)),
                _ => get_repo_json(&repo.url),
            },
        )
        .expect("Unable to write repo files");
    }
}

fn download(url: &str) -> Vec<u8> {
    let mut handle = Easy::new();
    let mut file: Vec<u8> = Vec::new();

    handle.url(url).unwrap();
    let _location = handle.follow_location(true);

    {
    let mut transfer = handle.transfer();
        transfer.write_function(|data| {
            file.extend_from_slice(data);
            Ok(data.len())
        }).unwrap();
        transfer.perform().unwrap();
    }
    file
}

fn download_file(output_file: &PathBuf, url: &str) -> std::io::Result<()> {
    create_dir_all(output_file.parent().unwrap()).expect("Couldn't create direcotires!");
    println!(
        "Downloading to {} from {}...",
        output_file.as_os_str().to_str().unwrap(),
        url
    );
    let mut file = File::create(output_file)?;
    file.write_all(download(url).as_slice())?;
    Ok(())
}

fn get_local_fonts() -> HashMap<String, Vec<FontFile>> {
    let mut results: HashMap<String, Vec<FontFile>> = HashMap::new();

    let source = SystemSource::new();
    let fonts = source.all_fonts().unwrap();

    for font in fonts {
        match font.load() {
            Ok(font_info) => {
                if let Handle::Path {
                    ref path,
                    font_index,
                } = font
                {
                    let metadata = fs::metadata(&path)
                        .expect("Unable to read metadata!");
                    let counter = results.entry(font_info.family_name())
                        .or_insert(Vec::new());
                    counter.push(FontFile {
                        variant: font_info.full_name(),
                        path: path.clone(),
                        system: true,
                        creation_date: 
                            if !(cfg!(target_os = "windows")) {
                                metadata.modified().unwrap()
                            } else {
                                metadata.modified().unwrap()
                            }
                    });
                }
            }
            Err(_) => {}
        }
    }
    results
}


fn get_populated_repos(
    repos: &Vec<repo::Repository>,
    repos_dir: &PathBuf
) -> HashMap<String, repo::FontsList> {
    let mut populated_repos: HashMap<String, repo::FontsList> = HashMap::new();

    for repo in repos{
        let repo_path = repos_dir.join(get_repo_as_file(&repo.name));
        if repo_path.exists() {
            populated_repos.insert(
                repo.name.clone(),
                serde_json::from_str(
                    &fs::read_to_string(&repo_path).expect("Couldn't find
                        specifiedrepository"),
                ).expect("Repository file bad formatted")
            );
        }
    }
    populated_repos
}

fn search_fonts(
    repos: &HashMap<String, repo::FontsList>,
    search_string: &str
) {
    let fonts: Vec<repo::Font> = Vec::new();

    for (repo_name, fonts_list) in repos.iter() {
        for font in fonts_list.items.iter() {
            if font.family.contains(search_string) {
                println!("{}/{}", repo_name, font.family);
                println!("Variants:");
                for variant in font.variants.iter() {
                    println!("  {}", variant);
                }
            }
        }
    }
}

fn download_fonts(
    repos: &HashMap<String, repo::FontsList>,
    download_dir: &PathBuf,
    selected_fonts: Vec<String>,
) {
    for (_repo_name, repo_fontlist) in repos.iter() {
        for selected_font in selected_fonts.iter() {
            for font in repo_fontlist.items.iter() {
                if &font.family == selected_font {
                    for (variant, url) in font.files.iter() {  
                        let extension: &str = url
                            .split(".")
                            .collect::<Vec<&str>>()
                            .last()
                            .expect("File extension not found!");
                        println!(
                            "Downloading {} from {}",
                            &format!(
                                "{}-{}.{}",
                                &font.family,
                                &variant,
                                &extension),
                            &url);

                        download_file(
                            &download_dir.join(&format!(
                                "{}-{}.{}",
                                &font.family,
                                &variant,
                                &extension)
                            ),
                            &url,
                        ).expect(&format!(
                                "Failed to download font {} {}",
                                font.family,
                                &variant)
                        );
                    }
                    break;
                }
            }
        }
    }
} 

fn load_font(
    repos: &HashMap<String, repo::FontsList>,
    download_dir: &PathBuf,
    selected_font: String,
) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::new();
    for (_repo_name, repo_fontlist) in repos.iter() {
        for font in repo_fontlist.items.iter() {
            if font.family == selected_font {
                bytes = download(font.files.values().collect::<Vec<&String>>().first().expect("Yield no results"));
                break;
            }
        }
    }
    bytes
}

fn remove_fonts(font_names: Vec<String>) {
    let local_fonts = get_local_fonts();
    for (family_name, font_list) in &local_fonts {
        for search_name in &font_names {
            if search_name == family_name {
                for font in font_list {
                    println!("Removing {}...", &font.path.display());
                    fs::remove_file(&font.path).expect("Unable to remove file");
                }
            }
        }
    }
}

fn main() {
    let font_catcher_dir = get_share_dir();

    let repos_dir = font_catcher_dir.join("repos");
    let install_dir = data_dir().expect("Couldn't solve for dir").join("fonts");

    let repos_file = font_catcher_dir.join("repos.conf");

    let repos: Vec<repo::Repository> = vec![
        repo::get_default_repos(),
        get_local_repos(&repos_file)
    ].into_iter().flatten().collect();

    let populated_repos: HashMap<String, repo::FontsList> = get_populated_repos(
        &repos,
        &repos_dir
    );

    let args: Vec<String> = args().collect();
    
    update_repos(&repos, &repos_dir);
    search_fonts(&populated_repos, "Roboto");

    /*
    for (family_name, fonts) in get_local_fonts() {
        for font in fonts {
            let date: DateTime<Utc> = font.creation_date.into();
            println!("{} -> {}: {:?}", family_name, font.variant, date);
        }
    }
    if (args.len() == 1) || ags[1] == "--version" || args[1] == "-v" {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        println!(
            "Copyright (C) {}, all rights reserved",
            env!("CARGO_PKG_AUTHORS")
        );
        println!("This is free software. It is licensed for use, modification and");
        println!("redistribution under the terms of the GNU Affero General Public License,");
        println!("version 3. <https://www.gnu.org/licenses/agpl-3.0.en.html>");
        println!("");
        println!("{}", env!("CARGO_PKG_DESCRIPTION"));
    } else if args[1] == "update-repos" {
        update_repos(local_repos, &repos_dir);
    } else if args.len() == 2 {
        if args[1] == "install"
            || args[1] == "remove"
            || args[1] == "search"
            || args[1] == "download"
        {
            println!("`{}` receives at least one argument", args[1]);
        } else {
            println!("`{}` command not found", args[1]);
        }
    } else {
        if args[1] == "install" {
            if args[2] == "--repo" {
                if args.len() > 4 {
                    download_fonts(&repos_dir, &install_dir, Some(&args[3]), args[4..].to_vec());
                } else {
                    println!("Missing fonts to install");
                }
            } else {
                download_fonts(&repos_dir, &install_dir, None, args[3..].to_vec());
            }
        } else if args[1] == "download" {
            if args[3] == "--repo" {
                if args.len() > 5 {
                    download_fonts(
                        &repos_dir,
                        &PathBuf::from(&args[2]),
                        Some(&args[4]),
                        args[5..].to_vec(),
                    );
                } else {
                    println!("Missing fonts or output directory to download");
                }
            } else {
                download_fonts(
                    &repos_dir,
                    &PathBuf::from(&args[2]),
                    None,
                    args[3..].to_vec(),
                );
            }
        } else if args[1] == "search" {
            if args[2] == "--repo" {
                if args.len() > 4 {
                    search_fonts(&repos_dir, Some(&args[3]), &args[4]);
                } else {
                    println!("Missing string to search");
                }
            } else {
                search_fonts(&repos_dir, None, &args[2]);
            }
        } else if args[1] == "remove" {
            remove_fonts(&install_dir, args[2..].to_vec());
        }
    }
    */
    /*
    Update repo files

    update_repos(default_repos, &repos_dir);
    update_repos(local_repos, &repos_dir);

    Search for fonts
    search_fonts(&repos_dir, None, "Agave");

    Download / Install fonts
    download_fonts(&repos_dir, &install_dir, None, vec!["Agave"]);
    download_fonts(
        &repos_dir,
        &install_dir,
        None,
        vec!["Roboto", "Hack", "ABeeZee"],
    );

    Remove fonts
    remove_fonts(&install_dir, vec!["ABeeZee", "Lmao"]);

    Create repo file from default repositories

    let repos_str = toml::to_string(&repos).unwrap();
    fs::write(config_file, repos_str).expect("Unable to write file");
    */
}
