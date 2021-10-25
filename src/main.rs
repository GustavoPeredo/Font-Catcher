use std::collections::HashMap;
use std::env::args;
use std::fs::{self, create_dir_all};

#[cfg(target_os = "windows")]
use std::os::windows::fs::MetadataExt;

#[cfg(target_os = "linux")]
use std::os::linux::fs::MetadataExt;

use std::path::PathBuf;
use std::process::Command;
use std::str;
use std::time;

use dirs::data_dir;

use font_kit::handle::Handle;
use font_kit::source::SystemSource;

use chrono::DateTime;
use chrono::offset::Utc;

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

fn get_local_repos(repo_file: PathBuf) -> Option<repo::Repositories> {
    if repo_file.exists() {
        let repo_data = fs::read_to_string(&repo_file)
            .expect("Unable to read repositories file");
        Some(toml::from_str(&repo_data)
             .expect("Failed to read repositories file")
        )
    } else {
        None
    }
}

fn get_repo_json(url: &str) -> String {
    if cfg!(target_os = "windows") {
        return "".to_string();
    }
    match str::from_utf8(
        &Command::new("curl")
            .arg(url)
            .output()
            .expect("Failed to execute process")
            .stdout,
    ) {
        Ok(v) => v.to_string(),
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    }
}

fn update_repos(repos: repo::Repositories, repos_dir: &PathBuf) {
    create_dir_all(repos_dir).expect("Couldn't create direcotires!");
    for repo in repos.repo.iter() {
        println!("Updating {}...", repo.name);
        fs::write(
            repos_dir.join(format!("{}{}", &repo.name, ".json")),
            match &repo.key {
                Some(key) => get_repo_json(&repo.url.replace("{API_KEY}", &key)),
                _ => get_repo_json(&repo.url),
            },
        )
        .expect("Unable to write repo files");
    }
}

fn family_in_repo(
    repos_dir: &PathBuf, 
    repo: &str, 
    search_string: &str
    )
    -> Vec<String> {
    let fonts_as_string =
        fs::read_to_string(repos_dir.join(&repo)).expect("Couldn't find specified repository");

    let mut results: Vec<String> = Vec::new();

    let fonts: repo::FontsList =
        serde_json::from_str(&fonts_as_string).expect("Repository file bad formatted");
    for font in fonts.items.iter() {
        if font.family.contains(&search_string) {
            results.push(font.family.to_owned());
        }
    }
    results
}

fn print_results(repo: &str, fonts: &Vec<String>) {
    if fonts.len() > 0 {
        for i in fonts.iter() {
            println!("{}/{}", &repo, &i);
        }
    } else {
        println!("No results found!");
    }
}

fn search_fonts(repos_dir: &PathBuf, repo: Option<&str>, search_string: &str) {
    match &repo {
        Some(repo) => {
            let results =
                family_in_repo(&repos_dir, &format!("{}{}", &repo, ".json"), search_string);
            print_results(&repo, &results);
        }
        _ => {
            let files = fs::read_dir(&repos_dir).expect("Folder not available");
            for repo in files {
                let results = family_in_repo(
                    &repos_dir,
                    &repo.as_ref().unwrap().file_name().to_str().unwrap(),
                    search_string,
                );
                print_results(
                    &repo
                        .as_ref()
                        .unwrap()
                        .path()
                        .file_stem()
                        .unwrap()
                        .to_str()
                        .unwrap(),
                    &results,
                );
            }
        }
    };
}

fn download_file(output_file: &PathBuf, url: &str) -> Result<std::process::Output, std::io::Error> {
    create_dir_all(output_file.parent().unwrap()).expect("Couldn't create direcotires!");
    println!(
        "Downloading to {} from {}...",
        output_file.as_os_str().to_str().unwrap(),
        url
    );
    Command::new("curl")
        .arg("-L") //Follow url
        .arg("-o")
        .arg(output_file.as_os_str().to_str().unwrap())
        .arg(url)
        .output()
}

fn files_in_repo(repos_dir: &PathBuf, repo: &str, search_string: &str) -> HashMap<String, String> {
    let fonts_as_string =
        fs::read_to_string(repos_dir.join(&repo)).expect("Couldn't find specified repository");

    let mut results: HashMap<String, String> = HashMap::new();

    let fonts: repo::FontsList =
        serde_json::from_str(&fonts_as_string).expect("Repository file bad formatted");
    for font in fonts.items.iter() {
        if font.family == search_string {
            for (i, j) in font.files.iter() {
                results.insert(i.to_string(), j.to_string());
            }
        }
    }
    results
}

fn download_fonts(
    repos_dir: &PathBuf,
    download_dir: &PathBuf,
    repo: Option<&str>,
    fonts: Vec<String>,
) {
    match &repo {
        Some(repo) => {
            for i in fonts.iter() {
                let results = files_in_repo(
                    &repos_dir, 
                    &format!("{}{}", &repo, ".json"),
                    i);

                for (variant, url) in results {
                    let extension: &str = url
                        .split(".")
                        .collect::<Vec<&str>>()
                        .last()
                        .expect("File extension not found!");
                    download_file(
                        &download_dir.join(&format!(
                                "{}-{}.{}",
                                i,
                                &variant,
                                extension)
                        ),
                        &url,
                    )
                    .expect(&format!("Failed to download font {}", i));
                }
            }
        }
        _ => {
            let files = fs::read_dir(&repos_dir).expect("Folder not available");
            for repo in files {
                for i in fonts.iter() {
                    let results = files_in_repo(
                        &repos_dir,
                        &repo.as_ref().unwrap().file_name().to_str().unwrap(),
                        i,
                    );
                    // Make this threaded
                    for (variant, url) in results {
                        let extension: &str = url
                            .split(".")
                            .collect::<Vec<&str>>()
                            .last()
                            .expect("File extension not found!");
                        download_file(
                            &download_dir.join(&format!(
                                    "{}-{}.{}",
                                    i,
                                    &variant,
                                    extension)
                            ),
                            &url,
                        )
                        .expect(&format!("Failed to download font {}", i));
                    }
                }
            }
        }
    };
}

fn _get_local_fonts(fonts_dir: &PathBuf) -> HashMap<String, Vec<PathBuf>> {
    let files = fs::read_dir(fonts_dir).expect("Folder no available");
    let mut results: HashMap<String, Vec<PathBuf>> = HashMap::new();
    for font in files {
        match font
            .as_ref()
            .unwrap()
            .path()
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .split("-")
            .collect::<Vec<&str>>()
            .first()
        {
            Some(font_name) => {
                let counter = results.entry(font_name.to_string())
                    .or_insert(Vec::new());
                counter.push(font.as_ref().unwrap().path());
            }
            None => {}
        }
    }
    results
}

fn get_local_fonts() -> HashMap<String, Vec<FontFile>> {
    /*let files = fs::read_dir(fonts_dir).expect("Folder not available");*/
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

    let default_repos: repo::Repositories = repo::get_default_repos();
    let local_repos: Option<repo::Repositories> = get_local_repos(repos_file);

    let args: Vec<String> = args().collect();
    
    for (family_name, fonts) in get_local_fonts() {
        for font in fonts {
            let date: DateTime<Utc> = font.creation_date.into();
            println!("{} -> {}: {:?}", family_name, font.variant, date);
        }
    }
    
    /*

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
        update_repos(default_repos, &repos_dir);
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
