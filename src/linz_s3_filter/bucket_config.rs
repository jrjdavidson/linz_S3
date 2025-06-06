// use log::debug;
use serde::Deserialize;
// use std::fs;
// use std::io::ErrorKind;
use std::sync::OnceLock;
// use std::time::Duration;

// The number of concurrent threads will be limited to the number of CPU cores multiplied by this constant

static CONCURRENCY_LIMIT_CPU_MULTIPLIER: OnceLock<usize> = OnceLock::new();

pub fn set_concurrency_multiplier(value: Option<usize>) {
    if let Some(value) = value {
        let _ = CONCURRENCY_LIMIT_CPU_MULTIPLIER.set(value);
    }
}

pub fn get_concurrency_limit() -> usize {
    let multiplier = *CONCURRENCY_LIMIT_CPU_MULTIPLIER.get().unwrap_or(&1);
    num_cpus::get() * multiplier
}

#[derive(Debug, Deserialize)]
pub struct ConfigFile {
    skip_signature: &'static str,
    region: &'static str,
}

static CONFIG_INSTANCE: OnceLock<ConfigFile> = OnceLock::new();
impl ConfigFile {
    pub fn global() -> &'static ConfigFile {
        CONFIG_INSTANCE.get().expect("Config is not initialized")
    }

    pub fn init() {
        let config = ConfigFile::default();
        CONFIG_INSTANCE
            .set(config)
            .expect("Config already initialized");
    }
}
impl Default for ConfigFile {
    fn default() -> Self {
        ConfigFile {
            skip_signature: "true",
            region: "ap-southeast-2",
        }
    }
}

// fn load_config() -> ConfigFile {
//     match fs::read_to_string("config.toml") {
//         Ok(config_str) => toml::from_str(&config_str).expect("Failed to parse config file"),
//         Err(e) if e.kind() == ErrorKind::NotFound => {
//             debug!("Config file not found, using default configuration.");
//             ConfigFile::default()
//         }
//         Err(e) => panic!("Failed to read config file: {:?}", e),
//     }
// }
pub fn get_opts() -> Vec<(&'static str, &'static str)> {
    let config = CONFIG_INSTANCE.get_or_init(ConfigFile::default);

    vec![
        ("skip_signature", config.skip_signature),
        ("region", config.region),
    ]
}
