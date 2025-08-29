use config::{Config, ConfigError, File};
use serde::{Deserialize, Serialize};
use std::{env, process};
use tracing::error;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HttpConfig {
    pub address: String,
    pub port: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Bpp {
    pub id: String,
    pub caller_uri: String,
    pub domain: String,
    pub version: String,
    pub ttl: String,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProviderDb {
    pub db_uri: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    debug: bool,
    pub use_mock_bpp_response: bool,
    pub bpp: Bpp,
    pub http: HttpConfig,
    pub provider_db: ProviderDb,
}

impl AppConfig {
    pub fn new() -> Result<Self, ConfigError> {
        let args: Vec<String> = env::args().collect();
        if args.len() < 2 {
            error!("Error: Configuration path not provided. Usage: cargo run -- <config_path>");
            process::exit(1);
        }
        let config_path = &args[1];

        let config = Config::builder()
            .add_source(File::with_name(&config_path))
            .build()?
            .try_deserialize()?;
        Ok(config)
    }
}
