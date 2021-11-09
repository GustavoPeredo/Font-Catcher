use std::collections::HashMap;
use std::env::args;
use std::fs::{self, create_dir_all, File};
use std::io::{Result, Write};
use std::process;

use std::path::PathBuf;
use std::str;
use std::time;

use dirs::data_dir;

#[cfg(unix)]
use dirs::font_dir;

//Windows workaround
#[cfg(target_os = "windows")]
fn windows_user_folder_fonts() -> Option<PathBuf> {
    return std::env::var_os("userprofile")
        .and_then(|u| if u.is_empty() { None } else {
            let f = PathBuf::from(u).join("AppData\\Local\\Microsoft\\Windows\\Fonts");
            if !f.is_dir() { None } else { Some(f) }
        })
}
#[cfg(target_os = "windows")]
use self::windows_user_folder_fonts as font_dir;

use font_kit::handle::Handle;
use font_kit::source::SystemSource;

use chrono::{DateTime, NaiveDate};
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

fn get_share_dir() -> Result<PathBuf> {
    let mut share = match data_dir() {
        Some(p) => p,
        _ => {
            println!("Couldn't solve for dir, using current");
            PathBuf::from("./")
        }
    };
    share.push("font-catcher");
    create_dir_all(&share)?;
    Ok(share)
}

fn get_local_repos(repo_file: &PathBuf) -> Result<Vec<repo::Repository>> {
    let repositories: repo::Repositories = 
        match toml::from_str(&fs::read_to_string(repo_file)?) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("error: {:#}", e);
                println!("Skipping reading from local repositories");
                repo::Repositories {
                    repo: Vec::<repo::Repository>::new()
                }
            }
    };
    Ok(repositories.repo)
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

fn get_local_fonts() -> HashMap<String, Vec<FontFile>> {
    let mut results: HashMap<String, Vec<FontFile>> = HashMap::new();

    let source = SystemSource::new();
    let fonts = source.all_fonts().unwrap();

    for font in fonts {
        match font.load() {
            Ok(font_info) => {
                if let Handle::Path {
                    ref path,
                    font_index: _,
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
                        creation_date: metadata.modified().unwrap()
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
) -> Result<HashMap<String, repo::FontsList>> {
    let mut populated_repos: HashMap<String, repo::FontsList> = HashMap::new();

    for repo in repos{
        let repo_path = repos_dir.join(get_repo_as_file(&repo.name));
        if repo_path.exists() {
            populated_repos.insert(
                repo.name.clone(),
                serde_json::from_str(&fs::read_to_string(&repo_path)?)?
            );
        }
    }
    Ok(populated_repos)
}

fn download_file(output_file: &PathBuf, url: &str) -> Result<()> {
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

pub fn update_repos(
    repos: &Vec<repo::Repository>,
    repos_dir: &PathBuf
) -> Result<()> {
    create_dir_all(repos_dir)?;
    for repo in repos.iter() {
        println!("Updating {}...", &repo.name);
        fs::write(
            repos_dir.join(get_repo_as_file(&repo.name)),
            match &repo.key {
                Some(key) => get_repo_json(&repo.url.replace("{API_KEY}", &key)),
                _ => get_repo_json(&repo.url),
            },
        )?;
    }
    Ok(())
}

pub fn search_fonts (
    repos: &HashMap<String, repo::FontsList>,
    search_string: &str
) -> Vec<String>  {
    let mut results = Vec::new();
    for (repo_name, fonts_list) in repos.iter() {
        for font in fonts_list.items.iter() {
            if font.family.to_lowercase().contains(&search_string.to_lowercase()) {
                println!("{}/{}", repo_name, font.family);
                println!("Variants:");
                results.push(font.family.clone());
                for variant in font.variants.iter() {
                    println!("  {}", variant);
                }
            }
        }
    }
    results
}

pub fn download_fonts(
    repos: &HashMap<String, repo::FontsList>,
    download_dir: &PathBuf,
    selected_fonts: Vec<String>,
) -> Result<()> {
    for (_repo_name, repo_fontlist) in repos.iter() {
        for selected_font in selected_fonts.iter() {
            for font in repo_fontlist.items.iter() {
                if font.family.eq_ignore_ascii_case(&selected_font) {
                    for (variant, url) in font.files.iter() {  
                        let extension: &str = url
                            .split(".")
                            .collect::<Vec<&str>>()
                            .last().unwrap();
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
                        )?;
                    }
                    break;
                }
            }
        }
    }
    Ok(())
} 

pub fn load_font(
    repos: &HashMap<String, repo::FontsList>,
    selected_font: String,
) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::new();
    for (_repo_name, repo_fontlist) in repos.iter() {
        for font in repo_fontlist.items.iter() {
            if font.family.eq_ignore_ascii_case(&selected_font) {
                bytes = download(
                    font.files.values().collect::<Vec<&String>>()
                        .first().unwrap()
                );
                break;
            }
        }
    }
    bytes
}

pub fn remove_fonts(font_names: Vec<String>) -> Result<()> {
    let local_fonts = get_local_fonts();
    for (family_name, font_list) in &local_fonts {
        for search_name in &font_names {
            if search_name.eq_ignore_ascii_case(&family_name) {
                for font in font_list {
                    println!("Removing {}...", &font.path.display());
                    fs::remove_file(&font.path)?;
                }
            }
        }
    }
    Ok(())
}

pub fn list_fonts(repos: &HashMap<String, repo::FontsList>) -> Vec<String> {
    let mut results: Vec<String> = Vec::new();
    for (_repo_name, fonts) in repos.iter() {
        for font in fonts.items.iter() {
            println!("{}", font.family);
            results.push(font.family.clone());
        }
    }
    results
}

pub fn check_for_font_updates(repos: &HashMap<String, repo::FontsList>) -> Vec<String> {
    let mut results: HashMap<String, DateTime<Utc>> = HashMap::new();
    for (local_family_name, local_fonts) in get_local_fonts() {
        let local_date: DateTime<Utc> = local_fonts.first().unwrap().creation_date.into();
        for (repo_name, repo_fonts) in repos.iter() {
            for repo_font in repo_fonts.items.iter() {
                if local_family_name == repo_font.family {
                    match &repo_font.lastModified {
                        Some(text_date) => {
                            let repo_date: DateTime<Utc> = DateTime::from_utc(NaiveDate::parse_from_str(text_date, "%Y-%m-%d").expect("Invalid format for font").and_hms(0,0,0), Utc);
                            if repo_date >= local_date {
                                results.entry(local_family_name.clone()).or_insert(local_date);
                            } else if results.contains_key(&local_family_name) {
                                results.remove(&local_family_name);
                            }
                        },
                        None => {},
                    }
                }
            }
        }
    }
    for key in results.keys() {
        println!("{}", key);
    }
    results.keys().cloned().collect()
}

fn print_version() {
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
}

fn run() -> Result<()> {
    let font_catcher_dir = get_share_dir()?;

    let repos_dir = font_catcher_dir.join("repos");
    let install_dir = font_dir().expect("Couldn't find font directory");

    let repos_file = font_catcher_dir.join("repos.conf");

    let mut repos: Vec<repo::Repository> = vec![
        repo::get_default_repos(),
        match get_local_repos(&repos_file) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("error: {:#}", e);
                println!("Skipping local repositories...");
                Vec::<repo::Repository>::new()
            }
        }
    ].into_iter().flatten().collect();
    
    let args: Vec<String> = args().collect();
    
    for i in 1..(args.len()) {
        if args[i].len() > 2 && &args[i][..2] == "--" {
            match &args[i][2..] {
                "repo" => {
                    for repo in 0..repos.len() {
                        if repos[repo].name != args[i + 1] {
                            repos.remove(repo);
                        }
                    }
                },
                _ => {}
            }
        }
    }

    let populated_repos: HashMap<String, repo::FontsList> = 
        match get_populated_repos(&repos, &repos_dir) {
            Ok(r) => r,
            Err(err) => {
                println!("error: {}", err);
                println!("continuing without repos");
                HashMap::new()
            }
        };
    
    if args.len() < 2 {
        print_version();
    } else {
        for i in 1..(args.len()) {
            match &args[i][..] {
                "version" => {
                    print_version();
                    break;
                },
                "update-repos" => {
                    update_repos(&repos, &repos_dir)?; 
                    break;
                },
                "install" => {
                    download_fonts(&populated_repos, &install_dir, args[i..].to_vec())?;
                    break;
                },
                "download" => {
                    download_fonts(&populated_repos, &PathBuf::from(&args[i+1]), args[i+1..].to_vec())?;
                    break;
                },
                "search" => {
                    search_fonts(&populated_repos, &args[i+1]);
                    break;
                },
                "remove" => {
                    remove_fonts(args[i..].to_vec())?;
                    break;
                },
                "update-check" => {
                    check_for_font_updates(&populated_repos);
                },
                "update" => {
                    download_fonts(&populated_repos, &install_dir, check_for_font_updates(&populated_repos))?;
                },
                "list" => {
                    list_fonts(&populated_repos);
                }
                _ => {
                    println!("{} is not a valid operation, skipping...", args[i]);
                }
            }
        }
    }
    Ok(())
}

fn main() {
    let result = run();
    match result {
        Ok(()) => {
            process::exit(0);
        }
        Err(err) => {
            eprintln!("error: {:#}", err);
            process::exit(1);
        }
    }
}
