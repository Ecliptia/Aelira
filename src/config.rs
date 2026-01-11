use std::fs;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub cluster: Option<ClusterConfig>,
}

#[derive(Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
}

#[derive(Deserialize)]
pub struct ClusterConfig {
    pub workers: Option<usize>,
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_data = fs::read_to_string("config.toml")?;
        let config: Config = toml::from_str(&config_data)?;
        Ok(config)
    }
}