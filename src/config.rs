use serde::{Deserialize, Serialize};
use serde_yaml;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub server_url: String,
    pub send_port: String,
    pub receive_port: String,
}

pub fn get_config() -> Config {
    let config_yaml = include_str!("config.yaml");
    let config: Config =
        serde_yaml::from_str(config_yaml).expect("Failed to parse Config YAML");
    println!("Configuration loaded: {:?}", config);
    config
}
