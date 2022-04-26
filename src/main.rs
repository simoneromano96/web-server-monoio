#![feature(type_alias_impl_trait)]
#![feature(async_closure)]
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use futures_util::future::BoxFuture;
use httparse::Request;
use monoio::io::{AsyncReadRent, AsyncWriteRentExt};
use monoio::net::{TcpListener, TcpStream};

// type AsyncHandler = Box<dyn Fn(monoio::net::TcpStream) -> BoxFuture<'static, std::io::Result<()>>>;
type AsyncHandler = fn(monoio::net::TcpStream) -> dyn Future<Output = std::io::Result<()>>;
// BoxFuture<'static, std::io::Result<()>>;
//  Box<dyn Fn(TcpStream) -> dyn Future<Output = std::io::Result<()>>>;

type PathHandler = HashMap<String, AsyncHandler>;

#[derive(Default)]
struct Router {
    routes: HashMap<String, PathHandler>,
}

impl Router {
    pub fn add(
        &mut self,
        method: &str,
        path: &str,
        // Pin<Box<dyn Future<Output=()> + 'a>>
        handler: AsyncHandler,
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

// fn by_length() -> impl Fn(TcpStream) -> Pin<Box<dyn Future<Output = std::io::Result<()>>>> {
//     move |stream| {
//         Box::pin(async move {
//             let response = b"HTTP/1.1 200 OK\r\n\r\n";
//             let (res, _) = stream.write_all(response.to_vec()).await;
//             res?;
//             Ok(())
//         })
//     }
// }

#[monoio::main]
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:3000").unwrap();
    let mut router = Router::default();

    fn add(x: i32, y: i32) -> i32 {
        x + y
    }

    let mut x = add(5, 7);

    type Binop = fn(i32, i32) -> i32;
    let bo: Binop = add;
    x = bo(5, 7);

    println!("listening");
    loop {
        let incoming = listener.accept().await;
        match incoming {
            Ok((stream, addr)) => {
                println!("accepted a connection from {}", addr);
                // let handler = Box::new(test_handler);
                // router.add("GET", "/test", by_length);

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
