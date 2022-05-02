mod parse;

use async_trait::async_trait;
use http_types::Method;
use monoio::{
    io::{AsyncReadRent, AsyncWriteRentExt},
    net::{TcpListener, TcpStream},
};
use parse::ParsedRequest;

use std::future::Future;
use std::{collections::HashMap, sync::Arc};

// type SyncHandler = Box<dyn Fn(ParsedRequest) -> Vec<u8>>;
// type AsyncHandler = fn(ParsedRequest) -> BoxFuture<'static, Vec<u8>>;
// Box<dyn Fn(ParsedRequest) -> dyn Future<Output = Vec<u8>>>;

type PathHandler = HashMap<String, Box<dyn Endpoint>>;

type Result = std::result::Result<Vec<u8>, ()>;

#[async_trait]
trait Endpoint: Send + Sync + 'static {
    /// Invoke the endpoint within the given context
    async fn call(&self, req: ParsedRequest) -> crate::Result;
}

// type DynEndpoint = dyn Endpoint;

#[async_trait]
impl<F, Fut> Endpoint for F
where
    F: Send + Sync + 'static + Fn(ParsedRequest) -> Fut,
    Fut: Future<Output = Result> + Send + 'static,
{
    async fn call(&self, req: ParsedRequest) -> crate::Result {
        let fut = (self)(req);
        let res = fut.await?;
        Ok(res.into())
    }
}

struct Router {
    routes: HashMap<Method, PathHandler>,
    not_found_handler: Box<dyn Endpoint>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
            not_found_handler: Box::new(not_found_handler),
        }
    }

    pub fn add<T>(
        &mut self,
        method: &Method,
        path: &str,
        // Pin<Box<dyn Future<Output=()> + 'a>>
        handler: T,
    ) where
        T: Endpoint,
    {
        match self.routes.get_mut(method) {
            Some(path_map) => {
                path_map.insert(path.to_string(), Box::new(handler));
            }
            None => {
                let mut path_map = PathHandler::new();
                path_map.insert(path.to_string(), Box::new(handler));
                self.routes.insert(*method, path_map);
            }
        }
    }

    pub async fn handle_route(&self, parsed_request: ParsedRequest) -> Result {
        let ParsedRequest { method, path, .. } = &parsed_request;
        let (_, handler) = self
            .routes
            .iter()
            .find(|(routes_method, _routes)| *routes_method == method)
            .ok_or(())?
            .1
            .iter()
            .find(|(route_path, _handler)| *route_path == path)
            .ok_or(())?;

        handler.call(parsed_request).await
    }
}

async fn not_found_handler(request: ParsedRequest) -> Result {
    Ok(b"HTTP/1.1 404 NOT FOUND\r\n\r\n".to_vec())
}

async fn test_handler(request: ParsedRequest) -> Result {
    Ok(b"HTTP/1.1 200 OK\r\n\r\n".to_vec())
}

// fn sync_handler(request: ParsedRequest) -> Vec<u8> {
//     b"HTTP/1.1 200 OK\r\n\r\n".to_vec()
// }

#[monoio::main]
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:3000").unwrap();
    let mut router = Router::new();

    // let h: SyncHandler = Box::new(sync_handler);
    // router.add(&Method::Get, "/test", h);
    router.add(&Method::Get, "/test", test_handler);

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

async fn handle_tcp(router: Arc<Router>, mut stream: TcpStream) -> std::io::Result<()> {
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
    let response = router.handle_route(request).await.unwrap();
    let (res, _) = write.write_all(response).await;
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
