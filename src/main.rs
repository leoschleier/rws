//! Multithreaded webserver.
use rws::{cli, connect, threading};
use std::net::TcpListener;
use tracing::info;

/// Entry point for running the multithreaded webserver.
fn main() {
    tracing_subscriber::fmt::init();

    let config = cli::Config::from_env();

    info!(
        config.address,
        config.port, config.root, "Web server started"
    );

    let address_port = format!("{}:{}", config.address, config.port);

    let listener = TcpListener::bind(&address_port);
    let listener = match listener {
        Ok(l) => l,
        Err(e) => {
            panic!("Could not bind {address_port}: {e}");
        }
    };

    let pool = threading::ThreadPool::new(4);

    // Iterate over connection attempts.
    // Here, a connection is the name for the full request-response cycle.
    for stream in listener.incoming() {
        let stream = match stream {
            Ok(s) => s,
            Err(e) => {
                info!("Failed connection attempt: {e}");
                continue;
            }
        };
        let r = config.root.clone();
        pool.execute(|| connect::handle_connection(stream, r));
    }
}
