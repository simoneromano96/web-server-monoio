mod handler;
mod handler_examples;
mod parse;
mod router;

use http_types::{Method, Response, Version};
use monoio::{
    io::{AsyncReadRent, AsyncWriteRentExt},
    net::{TcpListener, TcpStream},
};
use tracing::{error, info, instrument, Level};
use tracing_subscriber::FmtSubscriber;

use std::sync::Arc;

#[monoio::main]
async fn main() {
    // a builder for `FmtSubscriber`.
    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::TRACE)
        // completes the builder.
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let listener = TcpListener::bind("0.0.0.0:3000").unwrap();
    let mut router = router::Router::new();

    router.add(&Method::Get, "/test", handler_examples::test_handler);
    router.add(&Method::Get, "/json", handler_examples::another_handler);
    router.add(&Method::Post, "/json", handler_examples::body_handler);

    let router = Arc::new(router);

    info!("Listening on {:?}", listener.local_addr());

    loop {
        let incoming = listener.accept().await;
        match incoming {
            Ok((stream, addr)) => {
                info!("accepted a connection from {}", addr);
                // let handler = Box::new(actual_handler);
                // let h2: AsyncHandler = |stream| Box::pin(test_handler(stream));

                monoio::spawn(handle_tcp(router.clone(), stream));
            }
            Err(e) => {
                error!("accepted connection failed: {}", e);
                return;
            }
        }
    }
}

#[instrument]
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
    info!("Served request");
    Ok(())
}
