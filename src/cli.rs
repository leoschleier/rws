use dotenv;
use std::{env, path};

pub struct Parameters {
    pub address: String,
    pub port: String,
    pub root: String,
}

impl Parameters {
    pub fn from_env() -> Parameters {
        dotenv::dotenv().ok();

        let address =
            env::var("RWS_ADDRESS").unwrap_or("127.0.0.1".to_string());

        let port = env::var("RWS_PORT").unwrap_or("8080".to_string());

        let root = env::var("RWS_ROOT").unwrap_or(".".to_string());

        if !path::Path::new(&root).is_dir() {
            panic!("RWS_ROOT must be a valid directory. Got: {}", root);
        }

        Parameters {
            address,
            port,
            root,
        }
    }
}
