use regex::Regex;
use std::fs;
use std::io::{BufReader, prelude::*};
use std::net::TcpStream;
use std::path;

const ROOT_HTML: &str = "root.html";
const ERROR_404_NOT_FOUND_HTML: &str = "error/404.html";

/// Handle HTTP requests coming in over a `TCPStream`.
///
/// On each request, we serve static content in form of html.
pub fn handle_connection(mut stream: TcpStream, root: String) {
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

    let uri_path = path::Path::new(request_uri);
    let uri_resolved = resolve_uri(uri_path, &root);
    let file = path::Path::new(&uri_resolved);
    println!("Resolved uri: {:?}", file);

    let maybe_content = load_content(file);

    let (status_line, content_wrapper, content_type) =
        if let Some((Some(c), ctype)) = maybe_content {
            ("HTTP/1.1 200 OK", c, ctype)
        } else {
            println!("Error occurred when reading file {:?}", file);
            let f = format!("{root}/{ERROR_404_NOT_FOUND_HTML}");
            (
                "HTTP/1.1 404 NOT FOUND",
                StrOrBytes::Str(
                    fs::read_to_string(f).unwrap_or("".to_string()),
                ),
                String::from("text/html"),
            )
        };

    match content_wrapper {
        StrOrBytes::Bytes(b) => {
            send_response(&mut stream, status_line, &content_type, &b)
        }
        StrOrBytes::Str(s) => {
            send_response(&mut stream, status_line, &content_type, s.as_bytes())
        }
    };
}

fn resolve_uri(uri: &path::Path, root: &str) -> String {
    let uri_str;
    let mut extension = "";
    if uri == path::Path::new("/") {
        uri_str = ROOT_HTML;
    } else if uri.extension().is_none() {
        uri_str = uri.to_str().unwrap_or("");
        extension = ".html";
    } else {
        uri_str = uri.to_str().unwrap_or("");
    }

    format!("{root}/{uri_str}{extension}")
}

enum StrOrBytes {
    Str(String),
    Bytes(Vec<u8>),
}

fn load_content(file: &path::Path) -> Option<(Option<StrOrBytes>, String)> {
    match file.extension().and_then(|ext| ext.to_str()) {
        Some(ext) if matches!(ext, "css" | "html" | "txt") => Some((
            fs::read_to_string(file).ok().map(StrOrBytes::Str),
            format!("text/{ext}"),
        )),
        Some(ext) if matches!(ext, "json" | "wasm") => Some((
            fs::read(file).ok().map(StrOrBytes::Bytes),
            format!("application/{ext}"),
        )),
        Some(ext) if matches!(ext, "png") => Some((
            fs::read(file).ok().map(StrOrBytes::Bytes),
            format!("image/{ext}"),
        )),
        Some("svg") => Some((
            fs::read(file).ok().map(StrOrBytes::Bytes),
            "image/svg+xml".to_string(),
        )),
        Some("ico") => Some((
            fs::read(file).ok().map(StrOrBytes::Bytes),
            "image/vnd.microsoft.icon".to_string(),
        )),
        _ => None,
    }
}

fn send_response(
    stream: &mut TcpStream,
    status_line: &str,
    content_type: &str,
    content: &[u8],
) {
    let content_length = content.len();
    let header = format!(
        "{status_line}\r\n\
        Content-Type: {content_type}\r\n\
        Content-Length: {content_length}\r\n\r\n"
    );

    match stream.write_all(header.as_bytes()) {
        Ok(_) => (),
        Err(e) => eprintln!("Failed to send header: {e}"),
    }

    match stream.write_all(content) {
        Ok(_) => (),
        Err(e) => eprintln!("Failed to send content: {e}"),
    }
}
