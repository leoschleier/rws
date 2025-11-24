//! Multithreaded webserver.
use rws::{cli, connect, threading};
use std::net::TcpListener;

/// Entry point for running the multithreaded webserver.
fn main() {
    let parameters = cli::Parameters::from_env();

    println!("Serving static conent from {}", parameters.root);

    let address_port = format!("{}:{}", parameters.address, parameters.port);

    let listener = TcpListener::bind(&address_port);
    let listener = match listener {
        Ok(l) => l,
        Err(e) => {
            panic!("Could not bind {address_port}: {e}");
        }
    };

    println!("Listening to {}", address_port);

    let pool = threading::ThreadPool::new(4);

    // Iterate over connection attempts.
    // Here, a connection is the name for the full request-response cycle.
    for stream in listener.incoming() {
        let stream = match stream {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed connection attempt: {e}");
                continue;
            }
        };
        let r = parameters.root.clone();
        pool.execute(|| connect::handle_connection(stream, r));
    }
}
