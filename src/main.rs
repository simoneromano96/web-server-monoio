/// A echo example.
///
/// Run the example and `nc 127.0.0.1 50002` in another shell.
/// All your input will be echoed out.
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

async fn echo(stream: TcpStream) -> std::io::Result<()> {
    let mut buffer: Vec<u8> = Vec::with_capacity(8 * 1024);
    // loop {
    // read
    let (res, _buf) = stream.read(buffer).await;
    buffer = _buf;

    let res: usize = res?;
    if res == 0 {
        return Ok(());
    }

    println!("Request: \n{}", String::from_utf8_lossy(&buffer[..]));

    // write all
    let response = b"HTTP/1.1 200 OK\r\n\r\n";

    let (res, _buf) = stream.write_all(response.to_vec()).await;
    buffer = _buf;
    res?;

    // clear
    buffer.clear();
    Ok(())
    // }
}
