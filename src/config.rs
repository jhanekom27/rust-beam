use serde::{Deserialize, Serialize};
use serde_yaml;
use std::fs::File;
use std::io::Read;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub server_url: String,
    pub send_port: String,
    pub receive_port: String,
}

pub fn get_config() -> Config {
    // let config_path = "config.yaml";
    let config_yaml = include_str!("config.yaml");
    let config: Config =
        serde_yaml::from_str(config_yaml).expect("Failed to parse YAML");
    // read_config(config_path).expect("Failed to read configuration file.");
    println!("Configuration loaded: {:?}", config);
    config
}

// fn read_config(file_path: &str) -> Result<Config, serde_yaml::Error> {
//     let mut file = File::open(file_path).expect("Unable to open file");
//     let mut contents = String::new();
//     file.read_to_string(&mut contents)
//         .expect("Unable to read file");
//     let config: Config = serde_yaml::from_str(&contents)?;
//     Ok(config)
// }
