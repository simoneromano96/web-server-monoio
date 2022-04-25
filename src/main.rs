use httparse::Request;
use monoio::io::{AsyncReadRent, AsyncWriteRentExt};
use monoio::net::{TcpListener, TcpStream};

#[monoio::main]
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:3000").unwrap();
    println!("listening");
    loop {
        let incoming = listener.accept().await;
        match incoming {
            Ok((stream, addr)) => {
                println!("accepted a connection from {}", addr);
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
