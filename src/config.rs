use anyhow::Result;
use serde::Deserialize;
use std::fs;
use std::path::Path;

// Defining the ./config.json structure
#[derive(Debug, Deserialize)]
pub struct Config {
    pub num_of_cpus: u64,
    pub wallets_concat: Vec<String>,
    pub contracts_concat: Vec<String>,
}

pub fn load_config() -> Result<Config> {
    // Reading the ./config.json file
    let config_path = Path::new("./config.json");
    let config_data = fs::read_to_string(config_path)?;

    Ok(serde_json::from_str(&config_data)?)
}
