use std::env::args;
use std::collections::HashMap;
use std::fs::{read_dir, File};
use std::process::exit;
use std::io::{Result, Write};
use std::path::PathBuf;

use dirs::data_dir;
use serde_json::json;

mod lib;

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

struct Cli {
    location: Option<lib::Location>,
    command: String,
    repo: Option<String>,
    path: PathBuf,
    fonts: Vec<String>
}

fn run() -> Result<()> {
    let args: Vec<String> = args().collect();
    let mut clean_args: Vec<String> = Vec::new();

    let mut cli = Cli {
        location: None,
        command: "version".to_string(),
        repo: None,
        path: PathBuf::from("."),
        fonts: Vec::new()
    };
    
    let mut skip: bool = false;
    if args.len() > 1 {
        for i in 0..(args.len()) {
            match args[i].as_str() {
                "--repo" => {
                    cli.repo = Some(args[i + 1].clone()); 
                    skip = true;
                },
                "--user" => {
                    cli.location = Some(lib::Location::User);
                },
                "--path" => {
                    cli.path = PathBuf::from(&args[i + 1]);
                    skip = true;
                },
                "--system" => {
                    cli.location = Some(lib::Location::System);
                },
                _ => {
                    if !skip {
                        clean_args.push(args[i].clone());
                    }
                    skip = false;
                }
            }
        }
    } else {
        print_version();
        return Ok(());
    }
    cli.command = clean_args[1].clone();
    cli.fonts = clean_args[2..].to_vec();
    
    // Get directly from default repos
    // let fonts_list = lib::init()?;

    let font_catcher_dir = data_dir().unwrap().join("font-catcher");
    let repos_dir = font_catcher_dir.join("repos");
    let repos_file = font_catcher_dir.join("repos.conf");

    let local_repos_file: Vec<lib::Repository> = lib::generate_repos_from_file(&repos_file)?;

    let mut local_repos: HashMap<String, Vec<lib::RepoFont>> = HashMap::new();

    for file in read_dir(&repos_dir)? {
            let file = file.unwrap();
            match lib::generate_repo_font_list_from_file(&file.path()) {
                Ok(FontsList) => {
                    local_repos.insert(file.file_name().into_string().unwrap(), FontsList);
                },
                Err(_) => {
                    eprintln!("Error while reading repo...");
                }
            }
    }


    let mut fonts_list = lib::generate_fonts_list(local_repos.clone(), lib::generate_local_fonts(None).unwrap());

    match cli.command.as_str() {
        "version" => {
            print_version();
        },
        "update-repos" => {
            for r in local_repos_file.iter() {
                println!("Updating {}...", r.name); 
                let mut file = File::create(repos_dir.join(r.name.clone() + ".json"))?;
                file.write_all(
                    serde_json::to_string_pretty(&json!({
                        "kind": "webfonts#webfontList",
                        "items": &lib::generate_repo_font_list_from_url(
                            &r.url, r.key.clone()
                        )?
                    }))?.as_bytes()
                )?;
            }
        },
        "list-repos" => {
            for r in local_repos.keys() {
                println!("{}", r);
            }
        }
        "install" => {
            for font in cli.fonts.iter() {
                match fonts_list.get(font) {
                    Some(data) => {
                        data.install_to_user(cli.repo.as_deref(), true)?;
                    },
                    None => {
                        println!("{} not found anywhere!", font);
                        
                    }
                };
            }
        },
        "download" => {
            for font in cli.fonts.iter() {
                match fonts_list.get(font) {
                    Some(data) => {
                        data.download(cli.repo.as_deref(), &cli.path, true)?;
                    },
                    None => {
                        println!("{} not found anywhere!", font);
                        
                    }
                };
            }
        },
        "search" => {
            for font in cli.fonts.iter() {
                for (name, data) in &fonts_list {
                    if name.to_lowercase().contains(&font.to_lowercase()) &&
                        (match cli.repo {
                            Some(ref repo) => data.is_font_in_repo(&repo),
                            None => true
                        })
                    {
                        println!("\n{}:", &name);
                        println!("  Available on: {}", match data.get_repos_availability() { Some(r) => r.join(" "), None => "".to_string() });

                        println!("  User installed: {}", data.is_font_user_installed());
                        println!("  System installed: {}", data.is_font_system_installed());
                    }
                }
            }
        },
        "remove" => {
            for font in cli.fonts.iter() {
                match fonts_list.get(font) {
                    Some(data) => {
                        data.uninstall_from_user(true)?;
                    },
                    None => {
                        println!("{} not found anywhere!", font);
                    }
                };
            }
        },
        "check-for-updates" => {
            for (name, data) in &fonts_list {
                if cli.location == Some(lib::Location::System) {
                    match data.get_all_repos_with_update_system() {
                        Some(repos) => {
                            println!("Updates for {} available on:", name);
                            for r in repos.iter() {
                                println!("  {}", r);
                            }
                        },
                        None => {},
                    }
                } else {
                    match data.get_all_repos_with_update_user() {
                        Some(repos) => {
                            println!("Updates for {} available on:", name);
                            for r in repos.iter() {
                                println!("  {}", r);
                            }
                        },
                        None => {},
                    }
                }
            }
        },
        "update-all" => {
            for (name, data) in &fonts_list {
                if cli.location == Some(lib::Location::System) {
                    match data.get_all_repos_with_update_system() {
                        Some(repos) => {
                            data.install_to_user(Some(&repos[0]), true)?;
                        },
                        None => {},
                    }
                } else {
                    match data.get_all_repos_with_update_user() {
                        Some(repos) => {
                            data.install_to_user(Some(&repos[0]), true)?;
                        },
                        None => {},
                    }
                }
            }
        },
        "update" => {
            for font in cli.fonts.iter() {
                match fonts_list.get(font) {
                    Some(data) => {
                        if cli.location == Some(lib::Location::System) &&
                        data.is_update_available_system() {
                            data.install_to_user(
                                Some(
                                    &data.get_all_repos_with_update_system()
                                    .unwrap()[0]
                                ),
                                true
                            )?;
                        } else if data.is_update_available_user() {
                            data.install_to_user(
                                Some(
                                    &data.get_all_repos_with_update_user()
                                    .unwrap()[0]
                                ),
                                true
                            )?;
                        }
                    }
                    None => {
                        println!("{} not found anywhere!", font);
                    }
                }
            }
        }
        "list" => {
            for (name, data) in &fonts_list {
                if (cli.repo != None && data.is_font_in_repo(&cli.repo.as_ref().unwrap())) || cli.repo == None {
                    if cli.location == Some(lib::Location::System) &&
                        data.is_font_system_installed() {
                        println!("{}", name);
                    } else if cli.location == Some(lib::Location::User) &&
                        data.is_font_user_installed() {
                        println!("{}", name);
                    } else if cli.location == None{
                        println!("{}", name);
                    }
                }
            }
        }
        _ => {
            println!("{} is not a valid operation, skipping...", cli.command);
        }
    }
    Ok(())
}

fn main() {
    let result = run();
    match result {
        Ok(()) => {
            exit(0);
        }
        Err(err) => {
            eprintln!("error: {:#}", err);
            exit(1);
        }
    }
}
