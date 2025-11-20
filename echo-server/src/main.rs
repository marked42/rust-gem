use echo_server::server::Server;
use tokio;

#[tokio::main]
async fn main() {
    let server = Server::new("localhost:8080");
    server.listen();
}
