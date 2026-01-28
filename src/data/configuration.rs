use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[derive(Deserialize, Serialize, Default, Debug)]
pub struct Configuration {
    pub database: DatabaseConfig,
    pub family: HashMap<String, Family>,
}

impl Configuration {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config = std::fs::read_to_string("configuration.toml")?;
        let cfg: Self = toml::from_str(&config)?;
        Ok(cfg)
    }
}

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct DatabaseConfig {
    pub path: String,
}

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct Family {
    name: String,
    age: u32,
    #[serde(default)]
    children: Vec<String>,
}
