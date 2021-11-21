use std::env::args;
use std::collections::HashMap;
use std::fs::{read_dir};
use std::process::exit;
use std::io::Result;
use std::path::PathBuf;

use dirs::data_dir;

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
    /*
    let font_catcher_dir = data_dir().unwrap().join("font-catcher");
    let repos_dir = font_catcher_dir.join("repos");
    let repos_file = font_catcher_dir.join("repos.conf");

    let mut repos: Vec<lib::Repository> = vec![
        lib::get_default_repos(),
        match lib::generate_repos_from_file(&repos_file) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("error: {:#}", e);
                println!("Skipping local repositories...");
                Vec::<lib::Repository>::new()
            }
        }
    ].into_iter().flatten().collect();
    
        
    

    
    let repo_fonts_list: HashMap<String, Vec<lib::RepoFont>> = read_dir(&repos_dir)?.map(|file_name| {
        match &file_name {
            Ok(file_path) => {
                match lib::generate_repo_font_list_from_file(&file_path.path()) {
                    Ok(font_list) => (file_path.file_name().to_str().unwrap().to_string(), font_list),
                    Err(_) => {
                        eprintln!("error: while reading {}", file_path.path().display());
                        (String::new(), Vec::<lib::RepoFont>::new())
                    },
                }
            },
            Err(e) => {
                eprintln!("error:  {}", e);
                (String::new(), Vec::<lib::RepoFont>::new())
            }
        }
    }).collect();

    let mut fonts_list = lib::generate_fonts_list(repo_fonts_list, lib::generate_local_fonts(None).unwrap());
    */
    let fonts_list = lib::init()?;

    match cli.command.as_str() {
        "version" => {
            print_version();
        },
        "update-repos" => {
            //update_repos(&repos, &repos_dir)?; 
        },
        "install" => {
            for font in cli.fonts.iter() {
                match fonts_list.get(font) {
                    Some(f) => {
                        f.install_to_user(cli.repo.as_deref(), true)?;
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
                    Some(f) => {
                        f.download(cli.repo.as_deref(), &cli.path, true)?;
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
                    Some(f) => {
                        f.uninstall_from_user(true)?;
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
                            data.install_to_user(Some(&repos[0]), true);
                        },
                        None => {},
                    }
                } else {
                    match data.get_all_repos_with_update_user() {
                        Some(repos) => {
                            data.install_to_user(Some(&repos[0]), true);
                        },
                        None => {},
                    }
                }
            }
        },
        "list" => {
            //list_fonts(&populated_repos, true)?;
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
