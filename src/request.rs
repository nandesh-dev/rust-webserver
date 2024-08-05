use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Error, ErrorKind, Read, Result},
    net::{TcpListener, TcpStream},
    time::Duration,
};

#[derive(Debug)]
pub enum HttpMethod {
    GET,
    HEAD,
    POST,
    PUT,
    DELETE,
    CONNECT,
    OPTIONS,
    TRACE,
    OTHER(String),
}

fn parse_start_line(start_line: &String) -> Result<(HttpMethod, String, String)> {
    let mut parts = start_line.split_whitespace();

    let method = match parts.next() {
        Some(method) => match method {
            "GET" => HttpMethod::GET,
            "HEAD" => HttpMethod::HEAD,
            "POST" => HttpMethod::POST,
            "PUT" => HttpMethod::PUT,
            "OPTIONS" => HttpMethod::OPTIONS,
            "DELETE" => HttpMethod::DELETE,
            "CONNECT" => HttpMethod::CONNECT,
            "TRACE" => HttpMethod::TRACE,
            &_ => HttpMethod::OTHER(method.to_string()),
        },
        None => return Err(Error::new(ErrorKind::Other, "HTTP Method not found")),
    };

    let uri = match parts.next() {
        Some(uri) => uri,
        None => return Err(Error::new(ErrorKind::Other, "HTTP Method not found")),
    }
    .to_string();

    let version = match parts.next() {
        Some(version) => version,
        None => return Err(Error::new(ErrorKind::Other, "HTTP Method not found")),
    }
    .to_string();

    Ok((method, uri, version))
}

fn read_start_line(reader: &mut BufReader<&TcpStream>) -> Result<String> {
    let mut start_line = String::new();
    match reader.read_line(&mut start_line) {
        Ok(_) => Ok(start_line.trim_end().to_string()),
        Err(e) => Err(Error::new(
            ErrorKind::Other,
            format!("Error reading start line {}", e),
        )),
    }
}

fn parse_header_lines(header_lines: &Vec<String>) -> Result<HashMap<String, String>> {
    let mut headers: HashMap<String, String> = HashMap::new();

    for header_line in header_lines {
        match header_line.split_once(":") {
            Some(header) => {
                let (key, value) = header;
                headers.insert(key.trim().to_string(), value.trim().to_string());
            }
            None => {}
        }
    }

    Ok(headers)
}

fn read_header_lines(reader: &mut BufReader<&TcpStream>) -> Result<Vec<String>> {
    let mut header_lines: Vec<String> = Vec::new();
    loop {
        let mut header_line = String::new();
        match reader.read_line(&mut header_line) {
            Ok(_) => {
                if header_line == "\r\n" {
                    break;
                }

                header_lines.push(header_line.trim_end().to_string());
            }
            Err(e) => {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("Error reading header line, {}", e),
                ))
            }
        }
    }

    Ok(header_lines)
}

fn read_body(reader: &mut BufReader<&TcpStream>, content_length: u32) -> Result<String> {
    let mut body_buffer = vec![0; content_length as usize];
    match reader.read_exact(&mut body_buffer) {
        Ok(_) => Ok(String::from_utf8_lossy(&body_buffer).to_string()),
        Err(e) => Err(Error::new(
            ErrorKind::Other,
            format!("Error reading body, {}", e),
        )),
    }
}

#[derive(Debug)]
pub struct Request {
    method: HttpMethod,
    uri: String,
    version: String,
    headers: HashMap<String, String>,
    body: String,
}

impl Request {
    pub fn from_stream(stream: &TcpStream) -> Result<Request> {
        stream.set_read_timeout(Some(Duration::from_secs(5)))?;
        let mut reader = BufReader::new(stream);

        let start_line = read_start_line(&mut reader)?;
        let (method, uri, version) = parse_start_line(&start_line)?;

        let header_lines = read_header_lines(&mut reader)?;
        let headers = parse_header_lines(&header_lines)?;

        let mut content_length: u32 = 0;
        if let Some(content_length_string) = headers.get("Content-Length") {
            if let Ok(length) = content_length_string.parse() {
                content_length = length;
            }
        };

        let mut body = String::new();
        if content_length != 0 {
            body = read_body(&mut reader, content_length)?;
        };

        Ok(Request {
            method,
            uri,
            version,
            headers,
            body,
        })
    }
}
