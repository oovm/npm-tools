use task_system::TaskSystem;
use tokio::task::spawn_blocking;

#[test]
fn ready() {
    println!("it works!")
}

#[test]
fn task_channel() {
    let ts = TaskSystem::<u8>::default();
    ts.send(1);
    ts.send(2);
    ts.send(3);
    println!("checkpoint1: {:#?}", ts);
    ts.receive();
    spawn_blocking(async move || {
        ts.start(|task| {
            println!("{}", task);
            true
        })
    });

    println!("checkpoint2: {:#?}", ts);
    ts.send(4);
    println!("{:?}", ts.receive())
}
