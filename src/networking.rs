use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs};
use std::time::Duration;
use crate::error::{YppError, YppResult};

/// Hard caps so a remote peer cannot grow interpreter memory without bound.
pub const MAX_LINE_BYTES: usize = 64 * 1024;
pub const MAX_HOST_LEN: usize = 253;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(8);
const IO_TIMEOUT: Duration = Duration::from_secs(30);

/// Represents a networking object in the Y++ runtime.
pub enum NetObject {
    Client { stream: TcpStream },
    Server { listener: TcpListener },
    OutputStream { writer: BufWriter<TcpStream> },
    InputStream { reader: BufReader<TcpStream> },
    ConsoleReader,
    Keyboard,
    Frame,
    Dimensions { width: f64, height: f64 },
}

fn validate_host(host: &str, line: usize) -> YppResult<()> {
    let host = host.trim();
    if host.is_empty() || host.len() > MAX_HOST_LEN {
        return Err(YppError::invalid_network_host(line, host));
    }
    // Reject characters that should never appear in a host/IP and could
    // indicate injection if the value were ever passed to a shell.
    if host.chars().any(|c| c.is_whitespace() || c == '/' || c == '\\' || c == '\0') {
        return Err(YppError::invalid_network_host(line, host));
    }
    Ok(())
}

fn validate_port(port: u16, line: usize) -> YppResult<()> {
    if port == 0 {
        return Err(YppError::invalid_network_port(line, port));
    }
    Ok(())
}

fn apply_timeouts(stream: &TcpStream) {
    let _ = stream.set_read_timeout(Some(IO_TIMEOUT));
    let _ = stream.set_write_timeout(Some(IO_TIMEOUT));
    let _ = stream.set_nodelay(true);
}

/// Creates a new TCP client connection to the given host and port.
pub fn create_network(host: &str, port: u16, line: usize) -> YppResult<NetObject> {
    validate_host(host, line)?;
    validate_port(port, line)?;
    let addr = format!("{}:{}", host.trim(), port);
    let mut last_err = String::from("no addresses resolved");
    let resolved: Vec<SocketAddr> = match addr.to_socket_addrs() {
        Ok(iter) => iter.collect(),
        Err(e) => return Err(YppError::network_connect_failed(line, host, port, &e.to_string())),
    };
    if resolved.is_empty() {
        return Err(YppError::network_connect_failed(line, host, port, &last_err));
    }
    for sa in resolved {
        match TcpStream::connect_timeout(&sa, CONNECT_TIMEOUT) {
            Ok(stream) => {
                apply_timeouts(&stream);
                return Ok(NetObject::Client { stream });
            }
            Err(e) => last_err = e.to_string(),
        }
    }
    Err(YppError::network_connect_failed(line, host, port, &last_err))
}

/// Creates a new TCP server bound to `0.0.0.0` on the given port.
pub fn create_server(port: u16, line: usize) -> YppResult<NetObject> {
    validate_port(port, line)?;
    let addr = format!("0.0.0.0:{}", port);
    match TcpListener::bind(&addr) {
        Ok(listener) => {
            let _ = listener.set_nonblocking(false);
            Ok(NetObject::Server { listener })
        }
        Err(e) => Err(YppError::server_failed(line, port, &e.to_string())),
    }
}

/// Accepts an incoming connection on the given TCP listener.
pub fn server_accept(listener: &TcpListener, line: usize) -> YppResult<NetObject> {
    match listener.accept() {
        Ok((stream, _addr)) => {
            apply_timeouts(&stream);
            Ok(NetObject::Client { stream })
        }
        Err(e) => Err(YppError::stream_error(line, "accept", &e.to_string())),
    }
}

/// Creates a buffered output stream by cloning the given TCP stream.
pub fn get_output_stream(stream: &TcpStream, line: usize) -> YppResult<NetObject> {
    match stream.try_clone() {
        Ok(cloned) => {
            apply_timeouts(&cloned);
            Ok(NetObject::OutputStream {
                writer: BufWriter::new(cloned),
            })
        }
        Err(e) => Err(YppError::stream_error(line, "utf-8", &e.to_string())),
    }
}

/// Creates a buffered input stream by cloning the given TCP stream.
pub fn get_input_stream(stream: &TcpStream, line: usize) -> YppResult<NetObject> {
    match stream.try_clone() {
        Ok(cloned) => {
            apply_timeouts(&cloned);
            Ok(NetObject::InputStream {
                reader: BufReader::new(cloned),
            })
        }
        Err(e) => Err(YppError::stream_error(line, "readutf-8", &e.to_string())),
    }
}

/// Writes a message followed by a newline to the buffered writer and flushes.
pub fn stream_write(writer: &mut BufWriter<TcpStream>, msg: &str, line: usize) -> YppResult<()> {
    if msg.len() > MAX_LINE_BYTES {
        return Err(YppError::stream_error(
            line,
            "utf-8",
            "message exceeds 64KiB limit",
        ));
    }
    if let Err(e) = writeln!(writer, "{}", msg) {
        return Err(YppError::stream_error(line, "utf-8", &e.to_string()));
    }
    if let Err(e) = writer.flush() {
        return Err(YppError::stream_error(line, "utf-8", &e.to_string()));
    }
    Ok(())
}

/// Reads a single line from the buffered reader, trimming the trailing newline.
/// Returns an empty string on EOF. Rejects lines longer than [`MAX_LINE_BYTES`].
pub fn stream_read(reader: &mut BufReader<TcpStream>, line: usize) -> YppResult<String> {
    let mut buf = String::new();
    loop {
        let mut chunk = String::new();
        match reader.read_line(&mut chunk) {
            Ok(0) => {
                if buf.is_empty() {
                    return Ok(String::new());
                }
                break;
            }
            Ok(_) => {
                buf.push_str(&chunk);
                if buf.len() > MAX_LINE_BYTES {
                    return Err(YppError::stream_error(
                        line,
                        "readutf-8",
                        "incoming line exceeds 64KiB limit",
                    ));
                }
                if chunk.ends_with('\n') {
                    break;
                }
            }
            Err(e) => {
                return Err(YppError::stream_error(line, "readutf-8", &e.to_string()));
            }
        }
    }
    let trimmed = buf.trim_end_matches('\n').trim_end_matches('\r');
    Ok(trimmed.to_string())
}

/// Creates a console reader object.
pub fn create_console_reader() -> NetObject {
    NetObject::ConsoleReader
}
