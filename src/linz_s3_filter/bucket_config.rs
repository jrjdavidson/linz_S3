// use log::debug;
use serde::Deserialize;
// use std::fs;
// use std::io::ErrorKind;
use std::sync::OnceLock;
// use std::time::Duration;

// The number of concurrent threads will be limited to the number of CPU cores multiplied by this constant
pub const CONCURRENCY_LIMIT_CPU_MULTIPLIER: usize = 4;
// The number of concurrent threads for collections will be limited to this constant. Each collection will be allowed to spawn 1/this of the allowed concurrent threads.

// #[derive(Debug, Clone)]
// pub struct BackoffConfig {
//     pub init_backoff: Duration,
//     pub max_backoff: Duration,
//     pub base: f64,
// }

// #[derive(Debug, Clone)]
// pub struct RetryConfig {
//     pub backoff: BackoffConfig,
//     pub max_retries: usize,
//     pub retry_timeout: Duration,
// }

// const BACKOFF_CONFIG: BackoffConfig = BackoffConfig {
//     init_backoff: Duration::from_millis(500), // 500 milliseconds
//     max_backoff: Duration::from_secs(30),     // 30 seconds
//     base: 5.0,                                // Multiplier of 2.0
// };
// const RETRY_CONFIG: RetryConfig = RetryConfig {
//     max_retries: 3,
//     retry_timeout: Duration::from_secs(180), // 3 minutes
//     backoff: BACKOFF_CONFIG,
// };

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
