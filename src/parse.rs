use http_types::Method;
use std::{collections::HashMap, str::FromStr};
use thiserror::Error;
use tracing::trace;

#[derive(Error, Debug)]
pub enum ParseRequestError {
    #[error("Invalid method")]
    InvalidMethod,
    #[error("Incomplete HTTP request specification")]
    IncompleteRequest,
}

#[derive(Debug)]
pub struct ParsedRequest {
    pub method: Method,
    pub path: String,
    pub version: String,
    pub headers: Headers,
    pub body: Vec<u8>,
}

pub(crate) async fn parse_request(buffer: Vec<u8>) -> Result<ParsedRequest, ParseRequestError> {
    trace!("Parsing request");

    let lines: Vec<Vec<u8>> = buffer
        .into_iter()
        // .filter(|&byte| *byte != b'\n')
        .fold(Vec::new(), |mut acc, byte| {
            if acc.is_empty() {
                acc.push(Vec::new());
            }
            match byte {
                b'\r' => {
                    acc.push(Vec::new());
                }
                b'\n' => {
                    // noop
                }
                _ => {
                    if let Some(acc) = acc.last_mut() {
                        acc.push(byte);
                    }
                }
            };
            acc
        });

    let mut line_iter = lines.into_iter();

    let protocol_line = line_iter
        .next()
        .ok_or(ParseRequestError::IncompleteRequest)?;

    let HttpProtocol {
        method,
        path,
        version,
    } = get_protocol(protocol_line)?;

    let headers = parse_headers(&mut line_iter);

    let body = line_iter.flatten().collect();

    trace!("Parsed request successfully");

    Ok(ParsedRequest {
        headers,
        body,
        method,
        path,
        version,
    })
}

#[derive(Debug)]
struct HttpProtocol {
    method: Method,
    path: String,
    version: String,
}

fn get_protocol(line: Vec<u8>) -> Result<HttpProtocol, ParseRequestError> {
    let parsed_line = String::from_utf8_lossy(&line);
    let protocol: Vec<&str> = parsed_line.split(' ').collect();

    let (method, path, version) = (
        protocol
            .get(0)
            .ok_or(ParseRequestError::IncompleteRequest)?,
        protocol
            .get(1)
            .ok_or(ParseRequestError::IncompleteRequest)?,
        protocol
            .get(2)
            .ok_or(ParseRequestError::IncompleteRequest)?,
    );

    let method = Method::from_str(method).map_err(|_e| ParseRequestError::InvalidMethod)?;
    let path = path.to_string();
    let version = version.to_string();

    trace!(
        "Parsed request protocol; version = {:?} method = {:?} path = {:?}",
        &version,
        &method,
        &path
    );

    Ok(HttpProtocol {
        method,
        path,
        version,
    })
}

type Headers = HashMap<String, Vec<String>>;

fn parse_headers(lines: &mut std::vec::IntoIter<Vec<u8>>) -> Headers {
    lines
        .take_while(|line| !line.is_empty())
        .fold(HashMap::new(), |mut acc, header| {
            let parsed_line = String::from_utf8_lossy(&header);
            let header: Vec<&str> = parsed_line.split(": ").collect();
            if let (Some(header_key), Some(header_value)) = (header.get(0), header.get(1)) {
                if let Some(header_values) = acc.get_mut(*header_key) {
                    header_values.push(header_value.to_string());
                } else {
                    acc.insert(header_key.to_string(), vec![header_value.to_string()]);
                }
            };
            acc
        })
}
