use super::thread_pool::ThreadPool;
use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream, ToSocketAddrs},
};

pub struct Server {
    pool_threads_count: usize,
    listener: TcpListener,
}

impl Server {
    pub fn new_with_pool<A: ToSocketAddrs>(addr: A, pool_threads_count: usize) -> Server {
        let listener = TcpListener::bind(addr).unwrap();
        let addr = listener.local_addr().unwrap();
        println!("Server started on {}", addr);

        Server {
            listener,
            pool_threads_count,
        }
    }

    pub fn new<A: ToSocketAddrs>(addr: A) -> Server {
        Self::new_with_pool(addr, 10)
    }

    pub fn listen(&self) {
        let pool: ThreadPool = ThreadPool::new(self.pool_threads_count);

        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => {
                    pool.execute(|| {
                        Self::handle_connection(stream);
                    });
                }
                Err(e) => {
                    eprintln!("Connection failed: {}", e);
                    continue;
                }
            }
        }
    }

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
}
