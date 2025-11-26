# rws

Rust Web Server (RWS) is a minimal, multi-threaded web server that accepts requests via HTTP and serves static content.

## Configuration

The following environment variables can be used to configure RWS:

`RWS_ADDRESS` - Address to bind (default: "127.0.0.1")

`RWS_PORT` - Port to bind (default: "8080")

`RWS_ROOT` - Directory to serve content from (default: ".")

