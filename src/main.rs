use std::error::Error;
use std::io;
use std::io::prelude::*;
use std::net;

const BUFFER_SIZE: usize = 1024 * 1024; // 1 Mib
const CHUNK_SIZE: usize = 128;
const BUFFER_CAPACITY: usize = BUFFER_SIZE / CHUNK_SIZE;

fn handle_connection(mut stream: net::TcpStream) -> io::Result<()> {
    let mut buffer = Vec::with_capacity(BUFFER_CAPACITY);

    // TODO: Rewrite to BufReader
    // let mut reader = BufReader::with_capacity(BUFFER_CAPACITY, &stream);

    loop {
        if buffer.len() >= BUFFER_CAPACITY {
            println!("Request size limit is 1Mb, received more than that.");
            stream.write_all(b"HTTP/1.1 413 Payload Too Large\r\n")?;
            stream.shutdown(net::Shutdown::Both)?;

            return Ok(());
        }

        let mut chunk = [0u8; CHUNK_SIZE];
        let size = stream.read(&mut chunk)?;
        buffer.push(chunk);

        if size == 0 || size < CHUNK_SIZE {
            break;
        }
    }

    let buffer = buffer.concat();
    let raw_request = String::from_utf8_lossy(&buffer);
    let raw_request = raw_request.trim_matches(char::from(0));

    let mut lines = raw_request.split("\r\n");
    let status_line = lines.next().unwrap();
    let headers: Vec<&str> = lines.by_ref().take_while(|line| !line.is_empty()).collect();
    let body = lines.next().unwrap();

    assert_eq!(None, lines.next());

    println!("status: {:?}", status_line);
    println!("headers: {:?}", headers);
    println!("body: {:?}", body);

    stream.write(b"HTTP/1.1 204 No Content\r\n")?;
    stream.write(b"Connection: close\r\n")?;
    stream.write(b"Content-Length: 0\r\n")?;
    stream.flush()?;

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let socket: net::SocketAddrV4 = "127.0.0.1:7878".parse()?;
    let listener = net::TcpListener::bind(socket)?;

    println!("echo server started");

    for stream in listener.incoming() {
        let stream = stream?;
        println!("New connection: {}", stream.peer_addr()?);
        handle_connection(stream)?;
    }

    Ok(())
}
