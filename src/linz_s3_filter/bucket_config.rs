use log::debug;
use serde::Deserialize;
use std::fs;
use std::io::ErrorKind;
use std::time::Duration;

pub const CONCURRENCY_LIMIT_CPU_MULTIPLIER: usize = 1;
pub const CONCURRENCY_LIMIT_COLLECTIONS: usize = 3;

#[derive(Debug, Clone)]
pub struct BackoffConfig {
    pub init_backoff: Duration,
    pub max_backoff: Duration,
    pub base: f64,
}

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub backoff: BackoffConfig,
    pub max_retries: usize,
    pub retry_timeout: Duration,
}

const BACKOFF_CONFIG: BackoffConfig = BackoffConfig {
    init_backoff: Duration::from_millis(500), // 500 milliseconds
    max_backoff: Duration::from_secs(30),     // 30 seconds
    base: 5.0,                                // Multiplier of 2.0
};

const RETRY_CONFIG: RetryConfig = RetryConfig {
    max_retries: 3,
    retry_timeout: Duration::from_secs(180), // 3 minutes
    backoff: BACKOFF_CONFIG,
};

#[derive(Debug, Deserialize)]
struct ConfigFile {
    skip_signature: String,
    region: String,
}

impl Default for ConfigFile {
    fn default() -> Self {
        ConfigFile {
            skip_signature: "true".to_string(),
            region: "ap-southeast-2".to_string(),
        }
    }
}

fn load_config() -> ConfigFile {
    match fs::read_to_string("config.toml") {
        Ok(config_str) => toml::from_str(&config_str).expect("Failed to parse config file"),
        Err(e) if e.kind() == ErrorKind::NotFound => {
            debug!("Config file not found, using default configuration.");
            ConfigFile::default()
        }
        Err(e) => panic!("Failed to read config file: {:?}", e),
    }
}

pub fn get_opts() -> Vec<(&'static str, String)> {
    let config = load_config();
    vec![
        ("skip_signature", config.skip_signature),
        ("region", config.region),
        ("retry_config", format!("{:?}", RETRY_CONFIG)),
    ]
}
