use rust_gem::echo::Server;

fn main() {
    let server = Server::new("localhost:8000");
    server.listen();
    println!("after listen")
}
