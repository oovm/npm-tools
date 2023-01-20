use std::{thread::sleep, time::Duration};

use task_system::TaskSystem;

#[tokio::main]
pub async fn main() {
    let ts = TaskSystem::<u8>::default();
    ts.send(1);
    ts.send(2);
    ts.send(3);
    println!("checkpoint1: {:#?}", ts);
    ts.receive();
    ts.start(|task| {
        println!("{}", task);
        true
    });
    sleep(Duration::from_secs(1));
    println!("checkpoint2: {:#?}", ts);
    ts.send(4);
    println!("{:?}", ts.receive());
    ts.send(5);
}
