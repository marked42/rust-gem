use bytes::Bytes;
use mini_redis::{Connection, Frame};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::net::{TcpListener, TcpStream};

type SharedDb = Arc<Vec<Mutex<HashMap<String, Bytes>>>>;

fn new_shared_db(num_shards: usize) -> SharedDb {
    let mut db = Vec::with_capacity(num_shards);
    for _ in 0..num_shards {
        db.push(Mutex::new(HashMap::new()));
    }

    Arc::new(db)
}

fn get_shard_index(key: &str, db: &SharedDb) -> usize {
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    let db_hash = hasher.finish() as usize;
    db_hash % db.len()
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("localhost:6379").await.unwrap();
    println!("Listening");

    // let db = Arc::new(Mutex::new(HashMap::new()));
    let db = new_shared_db(10);

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let db = db.clone();

        println!("Accepted");
        tokio::spawn(async move {
            process(socket, db).await;
        });
    }
}

async fn process(socket: TcpStream, db: SharedDb) {
    use mini_redis::Command::{self, Get, Set};

    let mut connection = Connection::new(socket);

    while let Some(frame) = connection.read_frame().await.unwrap() {
        let response = match Command::from_frame(frame).unwrap() {
            Set(cmd) => {
                // TODO: shared db type
                let shard_index = get_shard_index(cmd.key(), &db);
                let mut db = db[shard_index].lock().unwrap();
                db.insert(cmd.key().to_string(), cmd.value().clone());
                Frame::Simple("OK".to_string())
            }
            Get(cmd) => {
                let shard_index = get_shard_index(cmd.key(), &db);
                let db = db[shard_index].lock().unwrap();
                if let Some(value) = db.get(cmd.key()) {
                    Frame::Bulk(value.clone())
                } else {
                    Frame::Null
                }
            }
            cmd => panic!("unimplemented {:?}", cmd),
        };

        connection.write_frame(&response).await.unwrap()
    }
}
