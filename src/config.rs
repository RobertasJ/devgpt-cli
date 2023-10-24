use std::io::{Read, Write};
use std::sync::{Arc, RwLock};
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use std::fs::File;
use once_cell::sync::Lazy;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub repo_location: Option<PathBuf>,
    pub project_summary: String,
}

const CONFIG_FILE: &str = "config.toml";

pub type AppConfig = Arc<RwLock<Config>>;

pub static CONFIG: Lazy<AppConfig> = Lazy::new(|| {
    println!("initializing config.");
    AppConfig::open()
});

impl Config {
    pub fn open() -> Self {
        let mut file = File::open(CONFIG_FILE).unwrap_or_else(|_| {
            let mut f = File::create(CONFIG_FILE).unwrap();
            f.write_all(toml::to_string_pretty(&Config::default()).unwrap().as_bytes()).unwrap();
            File::open(CONFIG_FILE).unwrap()
        });
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        toml::from_str(&contents).unwrap_or_default()
    }
    
    pub fn save(&self) {
        let mut file = File::create(CONFIG_FILE).unwrap();
        let contents = toml::to_string_pretty(self).unwrap();
        file.write_all(contents.as_bytes()).unwrap();
    }
}

pub trait ConfigTrait {
    fn save(&self);
    fn open() -> Self;
    fn repo_location(&self) -> Option<PathBuf>;
    fn project_summary(&self) -> String;
    fn set_repo_location(&self, repo_location: PathBuf);
    fn set_project_summary(&self, project_summary: String);
    
}

impl ConfigTrait for AppConfig {
    fn save(&self) {
        let config = self.read().unwrap();
        config.save();
    }

    fn open() -> Self {
        let config = Config::open();
        Arc::new(RwLock::new(config))
    }

    fn repo_location(&self) -> Option<PathBuf> {
        let config = self.read().unwrap();
        config.repo_location.clone()
    }

    fn project_summary(&self) -> String {
        let config = self.read().unwrap();
        config.project_summary.clone()
    }
    
    fn set_repo_location(&self, repo_location: PathBuf) {
        let mut config = self.write().unwrap();
        config.repo_location = Some(repo_location);
    }
    
    fn set_project_summary(&self, project_summary: String) {
        let mut config = self.write().unwrap();
        config.project_summary = project_summary;
    }
}
