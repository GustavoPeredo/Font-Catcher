use std::env::args;

mod lib;
/*
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

    let populated_repos: HashMap<String, Vec<repo::RepoFont>> = 
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
                    search_fonts(&populated_repos, &args[i+1], true)?;
                    break;
                },
                "remove" => {
                    remove_fonts(args[i..].to_vec())?;
                    break;
                },
                "update-check" => {
                    check_for_font_updates(&populated_repos)?;
                },
                "update" => {
                    download_fonts(&populated_repos, &install_dir, check_for_font_updates(&populated_repos)?)?;
                },
                "list" => {
                    list_fonts(&populated_repos, true)?;
                }
                _ => {
                    println!("{} is not a valid operation, skipping...", args[i]);
                }
            }
        }
    }
    Ok(())
}
*/
fn main() {

    /*let result = run();
    match result {
        Ok(()) => {
            process::exit(0);
        }
        Err(err) => {
            eprintln!("error: {:#}", err);
            process::exit(1);
        }
    }*/
}
