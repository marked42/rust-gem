use std::time::Duration;
use tokio::runtime::Builder;
use tokio::time::sleep;

fn main() {
    let runtime = Builder::new_multi_thread().worker_threads(1).enable_all().build().unwrap();
    
    let mut handles = Vec::with_capacity(10);
    for i in 0..10 {
        handles.push(runtime.spawn(my_bg_task(i)))
    }
    
    std::thread::sleep(Duration::from_millis(750));
    println!("Finish time consuming task.");
    
    for handle in handles {
        runtime.block_on(handle).unwrap();
    }
    
    println!("program finished");
}

async fn my_bg_task(i: u32) {
    let millis = 1000 - 50 * i; 
    println!("Task {} sleeping for {} ms.", i, millis);
    
    sleep(Duration::from_millis(millis as u64)).await;
    
    println!("Task {} stopping.", i);
}