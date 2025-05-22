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

/// Entry point for running the multithreaded webserver.
fn main() {
    dotenv().ok();

    let address = env::var("RWS_ADDRESS").unwrap();
    let port = env::var("RWS_PORT").unwrap();

    let address_port = format!("{}:{}", address, port);
    println!("Listening to {}", address_port);

    let listener = TcpListener::bind(address_port).unwrap();
    let pool = ThreadPool::new(4);

    // Iterate over connection attempts
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        pool.execute(|| handle_connection(stream));
    }
}

/// Handle HTTP requests coming in over a `TCPStream`.
///
/// On each request, we serve static content in form of html.
fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    let (status_line, filename) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "hello.html")
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
    };

    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();

    let response =
        format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
}
