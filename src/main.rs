#![feature(type_alias_impl_trait)]
#![feature(async_closure)]
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use httparse::Request;
use monoio::io::{AsyncReadRent, AsyncWriteRentExt};
use monoio::net::{TcpListener, TcpStream};

type AsyncHandler<'a> = impl Future<Output = std::io::Result<()>> + 'a;

type PathHandler<'a> = HashMap<String, AsyncHandler<'a>>;

#[derive(Default)]
struct Router<'a> {
    routes: HashMap<String, PathHandler<'a>>,
}

impl<'a> Router<'a> {
    pub fn add(
        &mut self,
        method: &str,
        path: &str,
        // Pin<Box<dyn Future<Output=()> + 'a>>
        handler: AsyncHandler<'a>,
    ) {
        match self.routes.get_mut(method) {
            Some(path_map) => {
                path_map.insert(path.to_string(), handler);
            }
            None => {
                let mut path_map = PathHandler::new();
                path_map.insert(path.to_string(), handler);
                self.routes.insert(method.to_string(), path_map);
            }
        }
    }
}

#[monoio::main]
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:3000").unwrap();
    let mut router = Router::default();

    println!("listening");
    loop {
        let incoming = listener.accept().await;
        match incoming {
            Ok((stream, addr)) => {
                println!("accepted a connection from {}", addr);
                router.add("GET", "/test", test_handler);

                monoio::spawn(echo(stream));
            }
            Err(e) => {
                println!("accepted connection failed: {}", e);
                return;
            }
        }
    }
}

async fn test_handler(stream: TcpStream) -> std::io::Result<()> {
    let response = b"HTTP/1.1 200 OK\r\n\r\n";
    let (res, _) = stream.write_all(response.to_vec()).await;
    res?;
    Ok(())
}

async fn not_found_handler(stream: TcpStream) -> std::io::Result<()> {
    let response = b"HTTP/1.1 404 NOT FOUND\r\n\r\n";
    let (res, _) = stream.write_all(response.to_vec()).await;
    res?;
    Ok(())
}

async fn echo(stream: TcpStream) -> std::io::Result<()> {
    let mut buffer = Vec::with_capacity(8 * 1024);

    // let (read, write) = stream.split();
    // let (res, _buf) = read.read(buffer).await;

    // read
    let (res, _buf) = stream.read(buffer).await;
    buffer = _buf;

    let res: usize = res?;
    if res == 0 {
        return Ok(());
    }

    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut req = httparse::Request::new(&mut headers);
    req.parse(&buffer).unwrap();

    match req {
        Request {
            method: Some("GET"),
            path: Some("/test"),
            ..
        } => {
            test_handler(stream).await?;
        }
        _ => {
            not_found_handler(stream).await?;
        }
    }

    // clear
    buffer.clear();
    Ok(())
    // }
}
