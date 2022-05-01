use http_types::Method;
use rayon::prelude::*;
use serde::Deserialize;
use simd_json::from_str;
use std::{collections::HashMap, str::FromStr};
use thiserror::Error;

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
    let mut lines: Vec<String> = Vec::with_capacity(buffer.len());
    buffer
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
        })
        .into_par_iter()
        .map(|line| String::from_utf8_lossy(&line).to_string())
        .collect_into_vec(&mut lines);

    let mut line_iter = lines.into_iter();
    let protocol_line = line_iter
        .next()
        .ok_or(ParseRequestError::IncompleteRequest)?;

    let HttpProtocol {
        method,
        path,
        version,
    } = get_protocol(&protocol_line)?;
    let headers = parse_headers(&mut line_iter);

    let body = line_iter
        .map(|body_line| body_line.as_bytes().to_vec())
        .flatten()
        .collect();

    Ok(ParsedRequest {
        headers,
        body,
        method,
        path,
        version,
    })
}

#[derive(Deserialize, Debug, PartialEq)]
struct TestJsonBody {
    test: String,
    hello: String,
}

fn parse_json_body(body: &mut str) -> TestJsonBody {
    println!("{body:#?}");
    let value: TestJsonBody = from_str(body).unwrap();
    println!("{value:#?}");
    value
}

#[derive(Debug)]
struct HttpProtocol {
    method: Method,
    path: String,
    version: String,
}

fn get_protocol(line: &str) -> Result<HttpProtocol, ParseRequestError> {
    let protocol: Vec<&str> = line.par_split(' ').collect();

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

    Ok(HttpProtocol {
        method,
        path,
        version,
    })
}

type Headers = HashMap<String, Vec<String>>;

fn parse_headers(lines: &mut std::vec::IntoIter<String>) -> Headers {
    lines
        .take_while(|line| !line.is_empty())
        .fold(HashMap::new(), |mut acc, header| {
            let header: Vec<&str> = header.split(": ").collect();
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
