//! Multithreaded webserver.
use dotenv::dotenv;
use rws::ThreadPool;
use std::{
    env, fs,
    io::{BufReader, prelude::*},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

const HELLO_HTML: &str = "html/hello.html";
const NOT_FOUND_HTML: &str = "html/404.html";

/// Entry point for running the multithreaded webserver.
fn main() {
    dotenv().ok();

    let address = env::var("RWS_ADDRESS").unwrap_or("127.0.0.1".to_string());

    let port = env::var("RWS_PORT").unwrap_or("8080".to_string());

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

    // Iterate over connection attempts
    for stream in listener.incoming() {
        let stream = match stream {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed connection attempt: {e}");
                continue;
            }
        };
        pool.execute(|| handle_connection(stream));
    }
}

/// Handle HTTP requests coming in over a `TCPStream`.
///
/// On each request, we serve static content in form of html.
fn handle_connection(mut stream: TcpStream) {
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

    let (status_line, filename) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", HELLO_HTML),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", HELLO_HTML)
        }
        _ => ("HTTP/1.1 404 NOT FOUND", NOT_FOUND_HTML),
    };

    let contents = fs::read_to_string(filename).unwrap_or("".to_string());
    let length = contents.len();

    let response =
        format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    match stream.write_all(response.as_bytes()) {
        Ok(_) => (),
        Err(e) => eprintln!("Failed to send response: {e}"),
    }
}
