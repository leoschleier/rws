//! Command line interface.
use dotenv;
use std::{env, path};

/// Web server configuration.
pub struct Config {
    /// Address to bind
    pub address: String,
    /// Port to bind
    pub port: String,
    /// Root directory to serve content from
    pub root: String,
}

impl Config {
    /// Creates a config from environment variables.
    pub fn from_env() -> Config {
        dotenv::dotenv().ok();

        let address =
            env::var("RWS_ADDRESS").unwrap_or("127.0.0.1".to_string());

        let port = env::var("RWS_PORT").unwrap_or("8080".to_string());

        let root = env::var("RWS_ROOT").unwrap_or(".".to_string());

        if !path::Path::new(&root).is_dir() {
            panic!("RWS_ROOT must be a valid directory. Got: {}", root);
        }

        Config {
            address,
            port,
            root,
        }
    }
}
