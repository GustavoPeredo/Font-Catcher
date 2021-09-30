use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Font {
    pub kind: Option<String>,
    pub family: String,
    pub variants: Vec<String>,
    pub subsets: Option<Vec<String>>,
    pub version: Option<String>,
    pub last_modified: Option<String>,
    pub files: HashMap<String, String>,
    pub commentary: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct FontsList {
    pub kind: String,
    pub items: Vec<Font>,
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

pub fn get_default_repos() -> Repositories {
    let repos: Repositories = Repositories {
        repo: vec![
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
        ],
    };
    repos
}
