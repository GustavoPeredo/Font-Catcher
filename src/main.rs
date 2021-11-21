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

fn run() -> Result<()> {
    let args: Vec<String> = args().collect();

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
                    //update_repos(&repos, &repos_dir)?; 
                    break;
                },
                "install" => {
                    for font in args[i + 1..].iter() {
                        match fonts_list.get(font) {
                            Some(f) => {
                                f.install_to_user(None, true)?;
                            },
                            None => {
                                println!("{} not found anywhere!", font);
                                
                            }
                        };
                    }
                    break;
                },
                "download" => {
                    for font in args[i + 2..].iter() {
                        match fonts_list.get(font) {
                            Some(f) => {
                                f.download(None, &PathBuf::from(args[2].clone()), true)?;
                            },
                            None => {
                                println!("{} not found anywhere!", font);
                                
                            }
                        };
                    }
                    break;
                },
                "search" => {
                    for font in args[i + 1..].iter() {
                        for (name, data) in &fonts_list {
                            if name.to_lowercase().contains(&font.to_lowercase()) {
                                println!("\n{}:", &name);
                                println!("  Available on: {}", match data.get_repos_availability() { Some(r) => r.join(" "), None => "".to_string() });

                                println!("  User installed: {}", data.is_font_user_installed());
                                println!("  System installed: {}", data.is_font_system_installed());
                            }
                        }
                    }
                    break;
                },
                "remove" => {
                    for font in args[i + 1..].iter() {
                        match fonts_list.get(font) {
                            Some(f) => {
                                f.uninstall_from_user(true)?;
                            },
                            None => {
                                println!("{} not found anywhere!", font);
                                
                            }
                        };
                    }
                    break;
                },
                "update-check" => {
                    break;
                    //check_for_font_updates(&populated_repos)?;
                },
                "update" => {
                    //download_fonts(&populated_repos, &install_dir, check_for_font_updates(&populated_repos)?)?;
                    break;
                },
                "list" => {
                    //list_fonts(&populated_repos, true)?;
                    break;
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
            exit(0);
        }
        Err(err) => {
            eprintln!("error: {:#}", err);
            exit(1);
        }
    }
}
