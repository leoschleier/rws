//! Handles TCP connections.
//!
//! This module provides functionality for:
//! - Parsing HTTP requests
//! - Serving static content from files
use regex::Regex;
use std::fs;
use std::io::{BufReader, prelude::*};
use std::net::TcpStream;
use std::path;
use tracing::{info, warn};

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
            warn!("Failed to read request line: {e}");
            return;
        }
        None => {
            warn!("No request line found");
            return;
        }
    };

    let re = Regex::new(r"^(GET) (.+) (HTTP/1\.1)$").unwrap();
    let captures = re.captures(&request_line);
    let request_line_captures = match captures {
        Some(c) => c,
        None => {
            warn!("Invalid request line: {request_line}");
            return;
        }
    };

    let request_uri = request_line_captures.get(2).unwrap().as_str();
    let file_pb = resolve_uri(&root, request_uri);
    let file_path = file_pb.as_path();
    info!(uri = request_uri, path = ?file_path, "Resolved request");

    let try_content = load_content(file_path);
    let (status_line, content, content_type) =
        if let Ok((content, content_type)) = try_content {
            ("HTTP/1.1 200 OK", content, content_type)
        } else {
            warn!(
                path=?file_path, error=try_content.unwrap_err(),
                "Error occurred when reading file"
            );
            let f = format!("{root}/{ERROR_404_NOT_FOUND_HTML}");
            (
                "HTTP/1.1 404 NOT FOUND",
                fs::read(f).unwrap_or("".as_bytes().to_vec()),
                "text/html".to_string(),
            )
        };

    send_response(&mut stream, status_line, &content_type, &content)
}

/// Finds file path associated with uri.
fn resolve_uri(root: &str, uri: &str) -> path::PathBuf {
    let mut uri_path;
    if uri == "/" {
        uri_path = path::Path::new(ROOT_HTML).to_path_buf();
    } else {
        uri_path = path::PathBuf::from(format!(".{uri}"));
    }

    debug_assert!(uri_path.is_relative());

    if uri_path.extension().is_none() {
        uri_path.set_extension("html");
    }

    let root_path = path::Path::new(root);
    root_path.join(uri_path).components().collect()
}

/// Loads content from file and determines its content type.
fn load_content(file: &path::Path) -> Result<(Vec<u8>, String), String> {
    let try_content = fs::read(file);
    if let Err(e) = try_content {
        return Err(e.to_string());
    }
    let content = try_content.unwrap();

    let content_type = match file.extension().and_then(|ext| ext.to_str()) {
        Some(ext) if matches!(ext, "css" | "html" | "txt") => {
            format!("text/{ext}")
        }
        Some(ext) if matches!(ext, "json" | "wasm") => {
            format!("application/{ext}")
        }
        Some(ext) if matches!(ext, "png") => format!("image/{ext}"),
        Some("svg") => "image/svg+xml".to_string(),
        Some("ico") => "image/vnd.microsoft.icon".to_string(),
        Some(ext) => return Err(format!("File type '{ext}' not supported")),
        None => return Err("Couldn't resolve file type".to_string()),
    };

    Ok((content, content_type))
}

/// Sends a response on a TCP stream.
///
/// # Arguments
///
/// * `stream` - TCP stream to send response on
/// * `status_line` - Status line of the response
/// * `content_type` - Content type of the response
/// * `content` - Content of the response
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
        Err(e) => {
            warn!(error = ?e, "Failed to send header");
            return;
        }
    }

    match stream.write_all(content) {
        Ok(_) => (),
        Err(e) => warn!(error = ?e, "Failed to send content"),
    }
}
