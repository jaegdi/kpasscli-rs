use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use anyhow::{Context, Result};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(rename = "database_path")]
    pub database_path: Option<String>,
    #[serde(rename = "default_output")]
    pub default_output: Option<String>,
    #[serde(rename = "password_file")]
    pub password_file: Option<String>,
    #[serde(rename = "password_executable")]
    pub password_executable: Option<String>,
    #[serde(default)]
    pub clipboard_timeout: Option<u64>,
    #[serde(skip)]
    pub config_file_path: String,
}

impl Config {
    pub fn load(config_path: &str) -> Result<Self> {
        let path = resolve_config_path(config_path);
        
        if !path.exists() {
            return Ok(Config {
                config_file_path: path.to_string_lossy().to_string(),
                ..Default::default()
            });
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {:?}", path))?;

        let mut config: Config = serde_yaml::from_str(&content)
            .with_context(|| "Failed to parse config file")?;
        
        config.config_file_path = path.to_string_lossy().to_string();
        Ok(config)
    }

    pub fn create_example(path: &str) -> Result<()> {
        let config = Config {
            database_path: Some("/path/to/your/database.kdbx".to_string()),
            default_output: Some("stdout".to_string()),
            password_file: Some("/path/to/your/password.txt".to_string()),
            password_executable: Some("[/path/to/your/]password_executable.sh".to_string()),
            clipboard_timeout: Some(15),
            config_file_path: "".to_string(),
        };

        let content = serde_yaml::to_string(&config)?;
        fs::write(path, content)?;
        Ok(())
    }
}

fn resolve_config_path(path: &str) -> PathBuf {
    if path.starts_with("~") {
        if let Some(home) = dirs::home_dir() {
            return home.join(path.trim_start_matches("~/"));
        }
    }
    PathBuf::from(path)
}
