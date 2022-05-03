mod handler;
mod parse;
mod router;

use futures_util::AsyncReadExt;
use handler::Handler;
use http_types::{Body, Method, Mime, Response, StatusCode, Version};
use monoio::{
    io::{AsyncReadRent, AsyncWriteRentExt},
    net::{TcpListener, TcpStream},
};
use parse::ParsedRequest;
use serde::{Deserialize, Serialize};
use simd_json::to_vec;

use std::{collections::HashMap, sync::Arc};

type PathHandler = HashMap<String, Box<dyn Handler>>;

async fn test_handler(_request: ParsedRequest) -> handler::HandlerResult {
    let mut res = Response::new(StatusCode::Ok);
    res.set_version(Some(Version::Http1_1));
    res.set_body("Hello, world!");
    res.append_header("Content-Type", "text/plain");
    Ok(res)
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
struct TestJsonBody {
    test: String,
    hello: String,
}

impl Into<Body> for TestJsonBody {
    fn into(self) -> Body {
        simd_json::to_vec(&self).unwrap().into()
    }
}

async fn another_handler(_request: ParsedRequest) -> handler::HandlerResult {
    let mut res = Response::new(StatusCode::Ok);
    res.set_version(Some(Version::Http1_1));
    res.set_body(TestJsonBody {
        test: "Hello".to_string(),
        hello: "This is test json body".to_string(),
    });
    res.set_content_type(http_types::mime::JSON);

    Ok(res)
}

// fn sync_handler(request: ParsedRequest) -> Vec<u8> {
//     b"HTTP/1.1 200 OK\r\n\r\n".to_vec()
// }

#[monoio::main]
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:3000").unwrap();
    let mut router = router::Router::new();

    // let h: SyncHandler = Box::new(sync_handler);
    // router.add(&Method::Get, "/test", h);
    router.add(&Method::Get, "/test", test_handler);
    router.add(&Method::Get, "/json", another_handler);

    let router = Arc::new(router);

    // fn add(x: i32, y: i32) -> i32 {
    //     x + y
    // }
    // let mut x = add(5, 7);
    // type Binop = fn(i32, i32) -> i32;
    // let bo: Binop = add;
    // x = bo(5, 7);

    println!("listening");
    loop {
        let incoming = listener.accept().await;
        match incoming {
            Ok((stream, addr)) => {
                println!("accepted a connection from {}", addr);
                // let handler = Box::new(actual_handler);
                // let h2: AsyncHandler = |stream| Box::pin(test_handler(stream));

                monoio::spawn(handle_tcp(router.clone(), stream));
            }
            Err(e) => {
                println!("accepted connection failed: {}", e);
                return;
            }
        }
    }
}

async fn response_to_buffer(response: &mut Response) -> Vec<u8> {
    let mut buffer = Vec::new();
    let version = response.version().unwrap_or(Version::Http1_1).to_string();
    let (status, canonical_reason) = (
        response.status().to_string(),
        response.status().canonical_reason(),
    );

    buffer.extend_from_slice(format!("{version} {status} {canonical_reason}\r\n").as_bytes());
    response.iter().for_each(|(header_name, header_values)| {
        header_values.iter().for_each(|header_value| {
            buffer.extend_from_slice(format!("{header_name}: {header_value}\r\n").as_bytes());
        });
    });
    if let Ok(body) = response.body_bytes().await {
        buffer.extend_from_slice(b"\r\n");
        buffer.extend_from_slice(&body);
    }
    buffer
}

async fn handle_tcp(router: Arc<router::Router>, mut stream: TcpStream) -> std::io::Result<()> {
    let mut buffer = Vec::with_capacity(8 * 1024);

    // Split stream into two components
    let (read, write) = stream.split();
    let (request, _buf) = read.read(buffer).await;

    // Empty request
    let res: usize = request?;
    if res == 0 {
        return Ok(());
    }

    // Move _buf into buffer for further inspection
    buffer = _buf;
    // Parse request
    let request = parse::parse_request(buffer).await.unwrap();

    let mut response = router.handle_route(request).await.unwrap();
    let buffer = response_to_buffer(&mut response).await;

    let (res, _) = write.write_all(buffer).await;
    res?;
    Ok(())

    // match request {
    //     ParsedRequest {
    //         method: Method::Get,
    //         ..
    //     } => test_handler(write).await,
    //     _ => not_found_handler(write).await,
    // }
    // match req {
    //     Request {
    //         method: Some("GET"),
    //         path: Some("/test"),
    //         ..
    //     } => test_handler(write).await,
    //     _ => not_found_handler(write).await,
    // }
    // Ok(())
}
