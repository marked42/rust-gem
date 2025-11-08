use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

fn handle_connection(mut stream: TcpStream) {
    println!("New connection: {}", stream.peer_addr().unwrap());
    let mut reader = BufReader::new(stream.try_clone().unwrap());

    loop {
        let mut message = String::new();
        match reader.read_line(&mut message) {
            Ok(0) => {
                println!("End");
                break;
            }
            Ok(bytes_read) => {
                println!("Received message {bytes_read} bytes : {message}");
                let _ = stream.write_all(message.as_bytes());

                message.clear();
            }
            Err(e) => {
                eprintln!("Error: {e}")
            }
        }
    }
}

fn main() {
    let listener = TcpListener::bind("localhost:0").unwrap();
    let addr = listener.local_addr().unwrap();
    println!("Server started on {}", addr);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // 创建一个线程处理一次请求
                // let handle =
                std::thread::spawn(|| {
                    handle_connection(stream);
                });
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
                continue;
            }
        }
    }
}
