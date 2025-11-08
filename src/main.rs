use std::{
    io::{BufRead, BufReader, Write},
    net::TcpListener,
};

fn main() {
    let listener = TcpListener::bind("localhost:8000").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
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
            Err(e) => {
                eprintln!("Connection failed: {}", e);
                continue;
            }
        }
    }
}
