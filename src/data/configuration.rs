use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct Configuration {
    pub database: DatabaseConfig,
    pub family: HashMap<String, Family>,
}

impl Configuration {
    pub fn load(configuration_path: String) -> Result<Self, Box<dyn std::error::Error>> {
        let config = std::fs::read_to_string(&configuration_path)?;
        let cfg: Self = toml::from_str(&config)?;
        Ok(cfg)
    }
}

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct DatabaseConfig {
    pub path: String,
}

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct Family {
    pub first_name: String,
    pub last_name: String,
    pub age: u32,
    #[serde(default)]
    pub children: Vec<String>,
}
