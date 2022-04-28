use http_types::Method;
use monoio::try_join;
use std::str::FromStr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseRequestError {
    #[error("Invalid method")]
    InvalidMethod,
    #[error("Incomplete HTTP request specification")]
    IncompleteRequest,
}

pub(crate) async fn parse_request(buffer: &[u8]) -> Result<(), ParseRequestError> {
    let lines: Vec<Vec<u8>> = buffer.into_iter().fold(Vec::new(), |mut acc, byte| {
        if acc.is_empty() {
            acc.push(Vec::new());
        }
        match byte {
            b'\r' => {
                acc.push(Vec::new());
            }
            _ => {
                if let Some(acc) = acc.last_mut() {
                    acc.push(*byte);
                }
            }
        };
        acc
    });

    lines.iter().for_each(|line| {
        println!("{line:?}");
        println!("{:#?}", String::from_utf8_lossy(&line));
    });

    // let (method, path, version) = get_protocol(&lines)?;
    // println!("{method} ___ {path} ___ {version}");

    let (protocol,) = try_join!(get_protocol(&lines))?;

    let (method, path, version) = protocol;
    println!("{method} ___ {path} ___ {version}");

    Ok(())
}

async fn get_protocol(lines: &Vec<Vec<u8>>) -> Result<(Method, String, String), ParseRequestError> {
    let protocol = lines
        .get(0)
        .ok_or(ParseRequestError::IncompleteRequest)?
        .iter()
        .fold(Vec::new(), |mut acc, byte| {
            if acc.is_empty() {
                acc.push(Vec::new());
            }
            match byte {
                b' ' => {
                    acc.push(Vec::new());
                }
                _ => {
                    if let Some(acc) = acc.last_mut() {
                        acc.push(*byte);
                    }
                }
            };
            acc
        });

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

    let method = Method::from_str(&String::from_utf8_lossy(method))
        .map_err(|_e| ParseRequestError::InvalidMethod)?;
    let path = String::from_utf8_lossy(path);
    let version = String::from_utf8_lossy(version);

    Ok((method, path.to_string(), version.to_string()))
}
