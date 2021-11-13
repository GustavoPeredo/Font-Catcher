use std::collections::HashMap;
use std::fs::{self, create_dir_all, File};
use std::io::{Result, Write};
use std::process;
use std::path::PathBuf;
use std::str;
use std::time::SystemTime;

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

use serde::{Deserialize, Serialize};
use serde_json;
use toml;

#[derive(Serialize, Deserialize, Clone)]
pub struct FontsList {
    pub kind: String,
    pub items: Vec<RepoFont>,
}

#[derive(Serialize, Deserialize)]
pub struct Repository {
    pub name: String,
    pub url: String,
    pub key: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Repositories {
    pub repo: Vec<Repository>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RepoFont {
    kind: Option<String>,
    family: String,
    variants: Vec<String>,
    subsets: Option<Vec<String>>,
    version: Option<String>,
    lastModified: Option<String>,
    files: HashMap<String, String>,
    commentary: Option<String>,
    creator: Option<String>,
}

#[derive(Clone)]
pub struct LocalFont{
    family: String,
    variants: Vec<String>,
    files: HashMap<String, PathBuf>,
    lastModified: SystemTime,
    system: bool
}

#[derive(Eq, PartialEq, Hash)]
pub enum Location {
    User,
    System,
}

pub struct Font{
    repo_font: HashMap<String, RepoFont>,
    local_font: HashMap<Location, LocalFont>,
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

pub fn get_default_repos() -> Vec<Repository> {
    vec![
        #[cfg(feature = "google_repo")]
        Repository {
            name: "Google Fonts".to_string(),
            url: "https://www.googleapis.com/webfonts/v1/webfonts?key={API_KEY}".to_string(),
            key: {
                const PASSWORD: &'static str = env!("GOOGLE_FONTS_KEY");
                Some(PASSWORD.to_string())
            },
        },
        Repository {
            name: "Open Font Repository".to_string(),
            url: "https://raw.githubusercontent.com/GustavoPeredo/open-font-repository/main/fonts.json".to_string(),
            key: None,
        }
    ]
}

pub fn generate_repos_from_str(repos_as_str: &str) -> Result<Vec<Repository>> {
    let repositories: Repositories = 
        match toml::from_str(&repos_as_str) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("error: {:#}", e);
                println!("Skipping reading from local repositories");
                Repositories {
                    repo: Vec::<Repository>::new()
                }
            }
    };
    Ok(repositories.repo)
}

pub fn generate_repos_from_file(
    repos_path: &PathBuf
) -> Result<Vec<Repository>> {
    Ok(generate_repos_from_str(&fs::read_to_string(repos_path)?)?)
}

pub fn generate_repo_font_list_from_str(
    font_list_as_str: &str
) -> Result<Vec<RepoFont>> {
    Ok(serde_json::from_str::<FontsList>(font_list_as_str)?.items)
}

pub fn generate_repo_font_list_from_file(
    repo_path: &PathBuf
) -> Result<Vec<RepoFont>> {
    Ok(generate_repo_font_list_from_str(&fs::read_to_string(repo_path)?)?)
}

pub fn generate_repo_font_list_from_url(
    repo_url: &str
) -> Result<Vec<RepoFont>> {
    Ok(generate_repo_font_list_from_str(
        match str::from_utf8(download(repo_url).as_slice()) {
            Ok(s) => s,
            Err(_) => ""
        }
    )?)
}

// Hopefully there is a better way to do this in the future
pub fn generate_local_fonts(
    location: Option<Location>
) -> Result<Vec<LocalFont>> {
    let mut user_results: HashMap<String, LocalFont> = HashMap::new();
    let mut sys_results: HashMap<String, LocalFont> = HashMap::new();

    let source = SystemSource::new();
    let fonts = source.all_fonts().unwrap();

    for font in fonts {
        match font.load() {
            Ok(font_info) => {
                if let Handle::Path {
                    ref path,
                    font_index: _,
                } = font {
                    match fs::metadata(&path) {
                        Ok(metadata) => {
                            let is_system = metadata.permissions().readonly();
                            if is_system {
                                let counter = sys_results.entry(font_info.family_name()).or_insert(LocalFont {
    family: font_info.full_name(),
    variants: Vec::new(),
    files: HashMap::new(),
    lastModified: metadata.modified()?,
    system: is_system
                                });
                                let variant_name = font_info.full_name().replace(&font_info.family_name(), "");
                                counter.variants.push(variant_name.clone());
                                counter.files.insert(variant_name.clone(), path.clone());
                            } else {
                                let counter = user_results.entry(font_info.family_name()).or_insert(LocalFont {
    family: font_info.family_name(),
    variants: Vec::new(),
    files: HashMap::new(),
    lastModified: metadata.modified()?,
    system: is_system
                                });
                                let variant_name = font_info.full_name().replace(&font_info.family_name(), "");
                                counter.variants.push(variant_name.clone());
                                counter.files.insert(variant_name.clone(), path.clone());

                            }
                        },
                        Err(_) => {}
                    }
                }
            },
            Err(_) => {}
        }
    }

    Ok(
        match location {
            Some(l) => {
                match l {
                    Location::User => user_results.values().cloned().collect(),
                    Location::System => sys_results.values().cloned().collect()
                }
            },
            None => vec![user_results.values().cloned().collect(), sys_results.values().cloned().collect::<Vec<LocalFont>>()].iter().flatten().cloned().collect()
        }
    )
}

pub fn generate_fonts_list(
    repos_font_lists: HashMap<String, Vec<RepoFont>>,
    local_fonts: Vec<LocalFont>
) -> HashMap<String, Font> {
    let mut result: HashMap<String, Font> = HashMap::new();

    for (repo_name, repo_fonts) in repos_font_lists.iter() {
        for repo_font in repo_fonts {
            let current_font = result.entry(repo_font.family.clone()).
                or_insert(Font {
                repo_font: HashMap::new(),
                local_font: HashMap::new()
            });
            current_font.repo_font.insert(repo_name.to_string(), repo_font.clone());
        }
    }
    for local_font in local_fonts {
        let current_font = result.entry(local_font.family.clone()).or_insert(Font {
            repo_font: HashMap::new(),
            local_font: HashMap::new()
        });
        if local_font.system {
            current_font.local_font.insert(Location::System, local_font.clone());
        } else {
            current_font.local_font.insert(Location::User, local_font.clone());
        }
    }
    result
}

impl Font {
    pub fn is_font_installed(&self) -> bool {
        if self.local_font.len() > 0 { true } else { false }
    }
    
    pub fn is_font_in_repo(&self,repo: &str) -> bool {
        match &self.repo_font.get(repo) {
            Some(_repo_font) => true,
            None => false
        }
    }

    pub fn get_local_user_variants(&self) -> Option<Vec<String>> {
        match &self.local_font.get(&Location::User) {
            Some(local_font) => Some(local_font.variants.clone()),
            None => None
        }
    }

    pub fn get_local_system_variants(&self) -> Option<Vec<String>> {
        match &self.local_font.get(&Location::System) {
            Some(local_font) => Some(local_font.variants.clone()),
            None => None
        }
    }

    pub fn get_repo_variants(&self, repo: &str) -> Option<Vec<String>> {
        match &self.repo_font.get(repo) {
            Some(repo_font) => Some(repo_font.variants.clone()),
            None => None
        }
    }
    
    pub fn get_local_user_files(&self) -> Option<HashMap<String, PathBuf>> {
        match &self.local_font.get(&Location::User) {
            Some(local_font) => Some(local_font.files.clone()),
            None => None
        }
    }

    pub fn get_local_system_files(&self) -> Option<HashMap<String, PathBuf>> {
        match &self.local_font.get(&Location::System) {
            Some(local_font) => Some(local_font.files.clone()),
            None => None
        }
    }

    pub fn get_repo_files(&self, repo: &str) -> Option<HashMap<String, String>> {
        match &self.repo_font.get(repo) {
            Some(repo_font) => Some(repo_font.files.clone()),
            None => None
        }
    }

    pub fn get_local_user_last_modified(&self) -> Option<DateTime<Utc>> {
        match &self.local_font.get(&Location::User) {
            Some(local_font) => Some(local_font.lastModified.into()),
            None => None
        }
    }

    pub fn get_local_system_last_modified(&self) -> Option<DateTime<Utc>> {
        match &self.local_font.get(&Location::System) {
            Some(local_font) => Some(local_font.lastModified.into()),
            None => None
        }
    }
    
    pub fn get_repo_last_modified(&self, repo: &str) -> Option<DateTime<Utc>> {
        match &self.repo_font.get(repo) {
            Some(repo_font) => {
                match &repo_font.lastModified {
                    Some(date) => {
                        let naive_date = NaiveDate::parse_from_str(
                            &date, "%Y-%m-%d"
                        );
                        match naive_date{
                            Ok(naive_date) => Some(DateTime::from_utc(
                                naive_date.and_hms(0,0,0), Utc)),
                            Err(_) => {
                                eprintln!("error: date not in %Y-%m-%d");
                                None
                            }
                        }
                    },
                    None => None
                }
            },
            None => None
        }
    }

    pub fn get_repo_subsets(&self, repo: &str) -> Option<Vec<String>> {
        match &self.repo_font.get(repo) {
            Some(repo_font) => {
                match &repo_font.subsets {
                    Some(i) => Some(i.clone()),
                    None => None
                }
            }
            None => None
        }
    }

    pub fn get_repo_version(&self, repo: &str) -> Option<String> {
        match &self.repo_font.get(repo) {
            Some(repo_font) => {
                match &repo_font.version {
                    Some(i) => Some(i.clone()),
                    None => None
                }
            }
            None => None
        }
    }

    pub fn get_repo_commentary(&self, repo: &str) -> Option<String> {
        match &self.repo_font.get(repo) {
            Some(repo_font) => {
                match &repo_font.commentary {
                    Some(i) => Some(i.clone()),
                    None => None
                }
            }
            None => None
        }
    }

    pub fn get_repo_creator(&self, repo: &str) -> Option<String> {
        match &self.repo_font.get(repo) {
            Some(repo_font) => {
                match &repo_font.creator {
                    Some(i) => Some(i.clone()),
                    None => None
                }
            }
            None => None
        }
    }
}

/*#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
*/