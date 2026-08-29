use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::{TcpListener, TcpStream};
use crate::error::{YppError, YppResult};

/// Represents a networking object in the Y++ runtime.
///
/// All networking objects are stored in the interpreter's object table
/// and referenced by ID (`usize`).
pub enum NetObject {
    Client { stream: TcpStream },
    Server { listener: TcpListener },
    OutputStream { writer: BufWriter<TcpStream> },
    InputStream { reader: BufReader<TcpStream> },
    ConsoleReader,
}

/// Creates a new TCP client connection to the given host and port.
///
/// On success, returns a `NetObject::Client` wrapping the connected stream.
pub fn create_network(host: &str, port: u16, line: usize) -> YppResult<NetObject> {
    let addr = format!("{}:{}", host, port);
    match TcpStream::connect(&addr) {
        Ok(stream) => Ok(NetObject::Client { stream }),
        Err(e) => {
            let err_msg = e.to_string();
            Err(YppError::network_connect_failed(line, host, port, &err_msg))
        }
    }
}

/// Creates a new TCP server bound to `0.0.0.0` on the given port.
///
/// On success, returns a `NetObject::Server` wrapping the listener.
pub fn create_server(port: u16, line: usize) -> YppResult<NetObject> {
    let addr = format!("0.0.0.0:{}", port);
    match TcpListener::bind(&addr) {
        Ok(listener) => Ok(NetObject::Server { listener }),
        Err(e) => {
            let err_msg = e.to_string();
            Err(YppError::server_failed(line, port, &err_msg))
        }
    }
}

/// Accepts an incoming connection on the given TCP listener.
///
/// On success, returns a `NetObject::Client` wrapping the accepted stream.
pub fn server_accept(listener: &TcpListener, line: usize) -> YppResult<NetObject> {
    match listener.accept() {
        Ok((stream, _addr)) => Ok(NetObject::Client { stream }),
        Err(e) => {
            let err_msg = e.to_string();
            Err(YppError::stream_error(line, "accept", &err_msg))
        }
    }
}

/// Creates a buffered output stream by cloning the given TCP stream.
///
/// Returns a `NetObject::OutputStream` wrapping a `BufWriter`.
pub fn get_output_stream(stream: &TcpStream, line: usize) -> YppResult<NetObject> {
    match stream.try_clone() {
        Ok(cloned) => Ok(NetObject::OutputStream {
            writer: BufWriter::new(cloned),
        }),
        Err(e) => {
            let err_msg = e.to_string();
            Err(YppError::stream_error(line, "utf-8", &err_msg))
        }
    }
}

/// Creates a buffered input stream by cloning the given TCP stream.
///
/// Returns a `NetObject::InputStream` wrapping a `BufReader`.
pub fn get_input_stream(stream: &TcpStream, line: usize) -> YppResult<NetObject> {
    match stream.try_clone() {
        Ok(cloned) => Ok(NetObject::InputStream {
            reader: BufReader::new(cloned),
        }),
        Err(e) => {
            let err_msg = e.to_string();
            Err(YppError::stream_error(line, "readutf-8", &err_msg))
        }
    }
}

/// Writes a message followed by a newline to the buffered writer and flushes.
pub fn stream_write(writer: &mut BufWriter<TcpStream>, msg: &str, line: usize) -> YppResult<()> {
    if let Err(e) = writeln!(writer, "{}", msg) {
        let err_msg = e.to_string();
        return Err(YppError::stream_error(line, "utf-8", &err_msg));
    }
    if let Err(e) = writer.flush() {
        let err_msg = e.to_string();
        return Err(YppError::stream_error(line, "utf-8", &err_msg));
    }
    Ok(())
}

/// Reads a single line from the buffered reader, trimming the trailing newline.
///
/// Returns an empty string on EOF.
pub fn stream_read(reader: &mut BufReader<TcpStream>, line: usize) -> YppResult<String> {
    let mut buf = String::new();
    match reader.read_line(&mut buf) {
        Ok(0) => Ok(String::new()),
        Ok(_) => {
            // Trim trailing newline characters (\n or \r\n)
            let trimmed = buf.trim_end_matches('\n').trim_end_matches('\r');
            Ok(trimmed.to_string())
        }
        Err(e) => {
            let err_msg = e.to_string();
            Err(YppError::stream_error(line, "readutf-8", &err_msg))
        }
    }
}

/// Creates a console reader object.
pub fn create_console_reader() -> NetObject {
    NetObject::ConsoleReader
}
