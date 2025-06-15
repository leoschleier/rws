//! Multithreaded webserver.
use dotenv::dotenv;
use regex::Regex;
use rws::ThreadPool;
use std::{
    env, fs,
    io::{BufReader, prelude::*},
    net::{TcpListener, TcpStream},
};

const ROOT_HTML: &str = "root.html";
const ERROR_404_NOT_FOUND_HTML: &str = "error/404.html";

/// Entry point for running the multithreaded webserver.
fn main() {
    dotenv().ok();

    let address = env::var("RWS_ADDRESS").unwrap_or("127.0.0.1".to_string());

    let port = env::var("RWS_PORT").unwrap_or("8080".to_string());

    let root = env::var("RWS_ROOT").unwrap_or(".".to_string());

    println!("Serving static conent from {}", root);

    let address_port = format!("{}:{}", address, port);

    let listener = TcpListener::bind(&address_port);
    let listener = match listener {
        Ok(l) => l,
        Err(e) => {
            panic!("Could not bind {address_port}: {e}");
        }
    };

    println!("Listening to {}", address_port);

    let pool = ThreadPool::new(4);

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
        let r = root.clone();
        pool.execute(|| handle_connection(stream, r));
    }
}

/// Handle HTTP requests coming in over a `TCPStream`.
///
/// On each request, we serve static content in form of html.
fn handle_connection(mut stream: TcpStream, root: String) {
    let buf_reader = BufReader::new(&stream);
    let request_line = match buf_reader.lines().next() {
        Some(Ok(line)) => line,
        Some(Err(e)) => {
            eprintln!("Failed to read request line: {e}");
            return;
        }
        None => {
            eprintln!("No request line found");
            return;
        }
    };

    let re = Regex::new(r"^(GET) (.+) (HTTP/1\.1)$").unwrap();

    let captures = re.captures(&request_line);

    let request_line_captures = match captures {
        Some(c) => c,
        None => {
            eprintln!("Invalid request line: {request_line}");
            return;
        }
    };

    let request_uri = request_line_captures.get(2).unwrap().as_str();

    println!("Request URI: {}", request_uri);

    let filename = match request_uri {
        "/" => format!("{root}/{ROOT_HTML}"),
        _ => format!("{root}/{request_uri}.html"),
    };

    let (status_line, content) = match fs::read_to_string(&filename) {
        Ok(content) => ("HTTP/1.1 200 OK", content),
        Err(e) => {
            eprintln!("Error occurred when reading file {}: {}", filename, e);
            let f = format!("{root}/{ERROR_404_NOT_FOUND_HTML}");
            (
                "HTTP/1.1 404 NOT FOUND",
                fs::read_to_string(f).unwrap_or("".to_string()),
            )
        }
    };

    let content_length = content.len();

    let response = format!(
        "{status_line}\r\nContent-Length: {content_length}\r\n\r\n{content}"
    );

    match stream.write_all(response.as_bytes()) {
        Ok(_) => (),
        Err(e) => eprintln!("Failed to send response: {e}"),
    }
}
