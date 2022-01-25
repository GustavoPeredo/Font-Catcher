use std::collections::HashMap;
use std::fs::{self, create_dir_all, File};
use std::io::{Result, Write};
use std::path::PathBuf;
use std::str;
use std::time::{SystemTime, UNIX_EPOCH};

use dirs::home_dir;

#[cfg(unix)]
use dirs::font_dir;

//Windows workaround
#[cfg(target_os = "windows")]
fn windows_user_folder_fonts() -> Option<PathBuf> {
    return std::env::var_os("userprofile").and_then(|u| {
        if u.is_empty() {
            None
        } else {
            let f = PathBuf::from(u).join("AppData\\Local\\Microsoft\\Windows\\Fonts");
            if !f.is_dir() {
                None
            } else {
                Some(f)
            }
        }
    });
}
#[cfg(target_os = "windows")]
use self::windows_user_folder_fonts as font_dir;

use font_kit::handle::Handle;
use font_kit::source::SystemSource;

use chrono::offset::Utc;
use chrono::{DateTime, NaiveDate};

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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RepoFont {
    kind: Option<String>,
    family: Option<String>,
    variants: Vec<String>,
    subsets: Option<Vec<String>>,
    version: Option<String>,
    lastModified: Option<String>,
    files: HashMap<String, String>,
    commentary: Option<String>,
    creator: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LocalFont {
    family: Option<String>,
    variants: Option<Vec<String>>,
    files: Option<HashMap<String, PathBuf>>,
    lastModified: Option<SystemTime>,
    installed: Option<bool>
}

#[derive(Eq, PartialEq, Hash, Debug, Clone)]
pub enum Location {
    User,
    System,
    Memory,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Font {
    family: String,
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
        transfer
            .write_function(|data| {
                file.extend_from_slice(data);
                Ok(data.len())
            })
            .unwrap();
        transfer.perform().unwrap();
    }
    file
}

fn download_file(output_file: &PathBuf, url: &str) -> Result<()> {
    create_dir_all(output_file.parent().unwrap())?;
    println!(
        "Downloading to {} from {}...",
        output_file.as_os_str().to_str().unwrap(),
        url
    );
    let mut file = File::create(output_file)?;
    file.write_all(download(url).as_slice())?;
    Ok(())
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
    let repositories: Repositories = match toml::from_str(&repos_as_str) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: {:#}", e);
            println!("Skipping reading from local repositories");
            Repositories {
                repo: Vec::<Repository>::new(),
            }
        }
    };
    Ok(repositories.repo)
}

pub fn generate_repos_from_file(repos_path: &PathBuf) -> Result<Vec<Repository>> {
    Ok(generate_repos_from_str(&fs::read_to_string(repos_path)?)?)
}

pub fn generate_repo_font_list_from_str(font_list_as_str: &str) -> Result<Vec<RepoFont>> {
    Ok(serde_json::from_str::<FontsList>(font_list_as_str)?.items)
}

pub fn generate_repo_font_list_from_file(repo_path: &PathBuf) -> Result<Vec<RepoFont>> {
    Ok(generate_repo_font_list_from_str(&fs::read_to_string(
        repo_path,
    )?)?)
}

pub fn generate_repo_font_list_from_url(
    repo_url: &str,
    key: Option<String>,
) -> Result<Vec<RepoFont>> {
    let repo_url = match key {
        Some(key) => repo_url.replace("{API_KEY}", &key),
        None => repo_url.to_string(),
    };
    Ok(generate_repo_font_list_from_str(
        match str::from_utf8(download(&repo_url).as_slice()) {
            Ok(s) => s,
            Err(_) => "",
        },
    )?)
}

pub fn init() -> Result<HashMap<String, Font>> {
    let local_fonts = generate_local_fonts(None)?;
    let default_repos = get_default_repos();
    let repo_fonts: HashMap<String, Vec<RepoFont>> = default_repos
        .iter()
        .map(|repo| {
            (
                repo.name.clone(),
                generate_repo_font_list_from_url(&repo.url, repo.key.clone()).unwrap(),
            )
        })
        .collect::<HashMap<String, Vec<RepoFont>>>();
    Ok(generate_fonts_list(repo_fonts, local_fonts))
}

pub fn generate_local_fonts(location: Option<Location>) -> Result<Vec<LocalFont>> {
    let fonts = SystemSource::new().all_families().unwrap();

    let results = fonts.iter().map(|font_family| {
        LocalFont {
            family: Some(font_family.to_string()),
            variants: None,
            files: None,
            lastModified: None,
            installed: None
        }
    }).collect::<Vec<LocalFont>>();
    Ok(results)
}

pub fn generate_fonts_list(
    repos_font_lists: HashMap<String, Vec<RepoFont>>,
    local_fonts: Vec<LocalFont>,
) -> HashMap<String, Font> {
    let mut result: HashMap<String, Font> = HashMap::new();

    for (repo_name, repo_fonts) in repos_font_lists.iter() {
        for repo_font in repo_fonts {
            let current_font = result.entry(repo_font.family.clone().unwrap()).or_insert(
                Font {
                    family: repo_font.family.clone().unwrap(),
                    repo_font: HashMap::new(),
                    local_font: HashMap::new(),
                }
            );
            current_font
                .repo_font
                .insert(repo_name.to_string(), repo_font.clone());
        }
    }

    for local_font in local_fonts {
        let local_font_format = Font {
            family: local_font.family.clone().unwrap(),
            repo_font: HashMap::new(),
            local_font: HashMap::from([
                (Location::System, local_font.clone()),
                (Location::User, local_font.clone()),
                (Location::Memory, local_font.clone())
            ]),
        };
        let current_font = result.entry(local_font.family.clone().unwrap()).or_insert(
            local_font_format
        );
        for location in [Location::User, Location::System, Location::Memory].iter() {
            current_font.local_font.insert(
                    location.clone(), local_font.clone(),
            );
        }
    }
    result
}

pub fn generate_local_font_from_handles(handles: &[Handle]) -> (Location, LocalFont) {
    let mut family_name = "".to_string();
    let mut variants: Vec<String> = Vec::new();
    let mut files: HashMap<String, PathBuf> = HashMap::new();
    let mut lastModified = None;

    let mut location = Location::Memory;
    
    for handle in handles.iter() {
        match handle.load() {
            Ok(font_info) => {
                family_name = font_info.family_name();

                let variant = match font_info.postscript_name() {
                    Some(postscript_name) => {
                        let mut var = postscript_name.replace(&family_name, "")
                            .replace(&family_name.replace(" ", ""), "")
                            .replace("-", " ");
                        if var.len() == 0 {
                            var = "Regular".to_string();
                        }
                        while variants.contains(&var) {
                            var = var + "-";
                        }
                        var
                    },
                    None => "Regular".to_string()
                };

                variants.push(variant.clone());

                match handle {
                    Handle::Path {ref path, font_index: _} => {
                        lastModified = Some(
                            match fs::metadata(&path) {
                                Ok(metadata) => {
                                    match metadata.modified() {
                                        Ok(time) => time,
                                        Err(_) => SystemTime::now()
                                    }
                                },
                                Err(_) => SystemTime::now()
                            }
                        );
                        location = if path.starts_with(home_dir().unwrap()) {
                            Location::User
                        } else {
                            Location::System
                        };

                        files.insert(
                            variant,
                            path.to_path_buf()
                        );
                    },
                    Memory => {
                        lastModified = Some(SystemTime::now());
                        location = Location::Memory;
                    }
                }
            },
            Err(_) => {}
        }
    }
    (
        location,
        LocalFont {
            family: Some(family_name),
            variants: {
                if !variants.is_empty() {
                    Some(variants)
                } else {
                    None
                }
            },
            files: {
                if !files.is_empty() {
                    Some(files)
                } else {
                    None
                }
            },
            lastModified: lastModified,
            installed: Some(true)
        }
    )
}

    macro_rules! create_fn {
        (
            $func_name:ident,
            $variable:ident,
            $default_return:expr,
            $return_type:ty
        ) => {
            fn $func_name(&mut self, location: &Location) -> $return_type {
                match self.local_font.get(location) {
                    Some(font) => {
                        match &font.$variable {
                            Some(value) => value.clone(),
                            None => {
                                match SystemSource::new().select_family_by_name(&self.family) {
                                    Ok(family_handle) => {
                                        let new_local_font = generate_local_font_from_handles(
                                            family_handle.fonts()
                                        );
                                        self.local_font.insert(
                                            new_local_font.0.clone(), new_local_font.1
                                        );
                                        if &new_local_font.0 == location {
                                            return self.$func_name(location);
                                        }
                                        $default_return
                                    },
                                    Err(_) => $default_return
                                }
                            }
                        }
                    },
                    None => $default_return
                }       
            }
        }
    }

impl Font {
    create_fn!(is_font_x_installed, installed, false, bool);
    create_fn!(get_local_x_variants, variants, Vec::new(), Vec<String>);
    create_fn!(get_local_x_files, files, HashMap::new(), HashMap<String, PathBuf>);
    create_fn!(get_local_x_last_modified, lastModified, SystemTime::now(), SystemTime);
    create_fn!(get_local_x_font_family, family, "".to_string(), String);

    pub fn is_font_system_installed(&mut self) -> bool {
        self.is_font_x_installed(&Location::System)
    }
    pub fn is_font_user_installed(&mut self) -> bool {
        self.is_font_x_installed(&Location::User)
    }
    pub fn is_font_memory_installed(&mut self) -> bool {
        self.is_font_x_installed(&Location::Memory)
    }

    pub fn is_font_installed(&mut self) -> bool {
        self.is_font_system_installed() || 
        self.is_font_user_installed() ||
        self.is_font_memory_installed()
    }

    pub fn get_local_system_variants(&mut self) -> Vec<String> {
        self.get_local_x_variants(&Location::System)
    }
    pub fn get_local_user_variants(&mut self) -> Vec<String> {
        self.get_local_x_variants(&Location::User)
    }
    pub fn get_local_memory_variants(&mut self) -> Vec<String> {
        self.get_local_x_variants(&Location::Memory)
    }

    pub fn get_local_system_files(&mut self) -> HashMap<String, PathBuf> {
        self.get_local_x_files(&Location::System)
    }
    pub fn get_local_user_files(&mut self) -> HashMap<String, PathBuf> {
        self.get_local_x_files(&Location::User)
    }
    pub fn get_local_memory_files(&mut self) -> HashMap<String, PathBuf> {
        self.get_local_x_files(&Location::Memory)
    }

    pub fn get_local_system_last_modified(&mut self) -> DateTime<Utc> {
        DateTime::<Utc>::from(
            self.get_local_x_last_modified(&Location::System)
        )
    }
    pub fn get_local_user_last_modified(&mut self) -> DateTime<Utc> {
        DateTime::<Utc>::from(
            self.get_local_x_last_modified(&Location::User)
        )
    }
    pub fn get_local_memory_last_modified(&mut self) -> DateTime<Utc> {
        DateTime::<Utc>::from(
            self.get_local_x_last_modified(&Location::Memory)
        )
    }

    pub fn get_local_system_font_family(&mut self) -> String {
        self.get_local_x_font_family(&Location::System).to_string()
    }
    pub fn get_local_user_font_family(&mut self) -> String {
        self.get_local_x_font_family(&Location::User).to_string()
    }
    pub fn get_local_memory_font_family(&mut self) -> String {
        self.get_local_x_font_family(&Location::Memory).to_string()
    }

    pub fn is_font_in_repo(&self, repo: &str) -> bool {
        match &self.repo_font.get(repo) {
            Some(_repo_font) => true,
            None => false,
        }
    }

    pub fn get_repos_availability(&self) -> Option<Vec<String>> {
        if self.repo_font.len() > 0 {
            Some(self.repo_font.keys().cloned().collect())
        } else {
            None
        }
    }

    pub fn get_repo_variants(&self, repo: &str) -> Option<Vec<String>> {
        match &self.repo_font.get(repo) {
            Some(repo_font) => Some(repo_font.variants.clone()),
            None => None,
        }
    }

    pub fn get_repo_files(&self, repo: &str) -> Option<HashMap<String, String>> {
        match &self.repo_font.get(repo) {
            Some(repo_font) => Some(repo_font.files.clone()),
            None => None,
        }
    }

    pub fn get_repo_last_modified(&self, repo: &str) -> Option<DateTime<Utc>> {
        match &self.repo_font.get(repo) {
            Some(repo_font) => match &repo_font.lastModified {
                Some(date) => {
                    let naive_date = NaiveDate::parse_from_str(&date, "%Y-%m-%d");
                    match naive_date {
                        Ok(naive_date) => {
                            Some(DateTime::from_utc(naive_date.and_hms(0, 0, 0), Utc))
                        }
                        Err(_) => {
                            eprintln!("error: date not in %Y-%m-%d");
                            None
                        }
                    }
                }
                None => None,
            },
            None => None,
        }
    }

    pub fn get_repo_family(&self, repo: &str) -> Option<String> {
        match &self.repo_font.get(repo) {
            Some(repo_font) => Some(repo_font.family.clone().unwrap()),
            None => None,
        }
    }

    pub fn get_repo_subsets(&self, repo: &str) -> Option<Vec<String>> {
        match &self.repo_font.get(repo) {
            Some(repo_font) => match &repo_font.subsets {
                Some(i) => Some(i.clone()),
                None => None,
            },
            None => None,
        }
    }

    pub fn get_repo_version(&self, repo: &str) -> Option<String> {
        match &self.repo_font.get(repo) {
            Some(repo_font) => match &repo_font.version {
                Some(i) => Some(i.clone()),
                None => None,
            },
            None => None,
        }
    }

    pub fn get_repo_commentary(&self, repo: &str) -> Option<String> {
        match &self.repo_font.get(repo) {
            Some(repo_font) => match &repo_font.commentary {
                Some(i) => Some(i.clone()),
                None => None,
            },
            None => None,
        }
    }

    pub fn get_repo_creator(&self, repo: &str) -> Option<String> {
        match &self.repo_font.get(repo) {
            Some(repo_font) => match &repo_font.creator {
                Some(i) => Some(i.clone()),
                None => None,
            },
            None => None,
        }
    }

    pub fn get_all_repos_with_update_user(&mut self) -> Option<Vec<String>> {
        let mut result: Vec<String> = Vec::new();
        let local_last_modified = &self.get_local_user_last_modified();
        match &self.get_repos_availability() {
            Some(repos) => {
                for repo in repos.iter() {
                    match &self.get_repo_last_modified(repo) {
                        Some(repo_last_modified) => {
                            if repo_last_modified > local_last_modified {
                                result.push(repo.to_string());
                            }
                        }
                        None => {}
                    }
                }
            }
            None => {}
        }
        if result.len() > 0 {
            Some(result)
        } else {
            None
        }
    }
    pub fn get_all_repos_with_update_system(&mut self) -> Option<Vec<String>> {
        let mut result: Vec<String> = Vec::new();
        let local_last_modified = &self.get_local_system_last_modified();
        match &self.get_repos_availability() {
            Some(repos) => {
                for repo in repos.iter() {
                    match &self.get_repo_last_modified(repo) {
                        Some(repo_last_modified) => {
                            if repo_last_modified > local_last_modified {
                                result.push(repo.to_string());
                            }
                        }
                        None => {}
                    }
                }
            }
            None => {}
        }
        if result.len() > 0 {
            Some(result)
        } else {
            None
        }
    }

    pub fn is_update_available_user(&mut self) -> bool {
        match self.get_all_repos_with_update_user() {
            Some(_repos) => true,
            None => false,
        }
    }
    pub fn is_update_available_system(&mut self) -> bool {
        match self.get_all_repos_with_update_system() {
            Some(_repos) => true,
            None => false,
        }
    }

    pub fn get_first_available_repo(&self) -> Option<String> {
        let repos = &self.get_repos_availability();
        match repos {
            Some(repos) => Some(repos.first().unwrap().to_string()),
            None => None,
        }
    }

    pub fn uninstall_from_user(&mut self, output: bool) -> Result<()> {
        for (_name, file) in self.get_local_user_files(){
            if output {
                println!("Removing {}...", &file.display());
            }
            fs::remove_file(&file)?;
        }
        self.local_font.insert(
            Location::User,
            LocalFont {
                family: None,
                variants: None,
                files: None,
                lastModified: None,
                installed: Some(false)
            }
        );
        Ok(())
    }

    pub fn uninstall_from_system(&mut self, output: bool) -> Result<()> {
        for (_name, file) in self.get_local_system_files() {
            if output {
                println!("Removing {}...", &file.display());
            }
            fs::remove_file(&file)?;
        }
        self.local_font.insert(
            Location::System,
            LocalFont {
                family: None,
                variants: None,
                files: None,
                lastModified: None,
                installed: Some(false)
            }
        );
        Ok(())
    }

    pub fn download(
        &self,
        repo: Option<&str>,
        download_path: &PathBuf,
        output: bool,
    ) -> Result<()> {
        let repos = self.get_first_available_repo();
        let repo = match repo {
            Some(repo) => repo,
            None => match &repos {
                Some(repo) => repo,
                None => "",
            },
        };
        match self.get_repo_files(repo) {
            Some(files) => {
                for (variant, file) in files {
                    let extension: &str = file.split(".").collect::<Vec<&str>>().last().unwrap();

                    if output {
                        println!(
                            "Downloading {} from {}",
                            &format!(
                                "{}-{}.{}",
                                &self.get_repo_family(repo).unwrap(),
                                &variant,
                                &extension
                            ),
                            &file
                        );
                    }
                    download_file(
                        &download_path.join(&format!(
                            "{}-{}.{}",
                            &self.get_repo_family(repo).unwrap(),
                            &variant,
                            &extension
                        )),
                        &file,
                    )?;
                }
            }
            None => {}
        }
        Ok(())
    }

    pub fn output_paths(
        &self,
        repo: Option<&str>,
        path: &PathBuf
    ) -> Vec<PathBuf> {
        let repos = self.get_first_available_repo();
        let repo = match repo {
            Some(repo) => repo,
            None => match &repos {
                Some(repo) => repo,
                None => "",
            },
        };

        let mut results: Vec<PathBuf> = Vec::new();

        match self.get_repo_files(repo) {
            Some(files) => {
                for (variant, file) in files {
                    let extension: &str = file.split(".").collect::<Vec<&str>>().last().unwrap();
                    results.push(
                        path.join(&format!(
                            "{}-{}.{}",
                            &self.get_repo_family(repo).unwrap(),
                            &variant,
                            &extension
                        ))
                    );
                }
            }
            None => {}
        }

        results
    }

    pub fn install_to_user(&mut self, repo: Option<&str>, output: bool) -> Result<()> {
        let install_dir = font_dir().unwrap();

        self.download(repo.clone(), &install_dir, output)?;

        let new_local_font = generate_local_font_from_handles(
            &self.output_paths(repo, &install_dir).iter().map(
                |path| {
                    Handle::from_path(path.to_path_buf(), 0)
                }).collect::<Vec<Handle>>()
        );
        self.local_font.insert(
            new_local_font.0.clone(), new_local_font.1
        );

        Ok(())
    }
}
