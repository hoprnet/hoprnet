pub mod channel;
pub mod data;
pub mod delay;
pub mod errors;
pub mod mixer;

// Notes
//
// recv() in std channels is the only function that blocks, blocking in ccoperative concurrency
// block all other execution paths.
//
// ## Context usage
// - contains waker
// ```rust
//     let waker = cx.waker();
//     waker.wake();
// ```

// #[tokio::main]
// async fn main() {
//     let (tx, mut rx) = channel();
//     // let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

//     let tx1 = tx.clone();
//     tokio::spawn(async move {
//         tokio::time::sleep(Duration::from_secs(4)).await;
//         tx1.send(1).expect("channel should be open");
//     });

//     let tx2 = tx;
//     tokio::spawn(async move {
//         tokio::time::sleep(Duration::from_secs(2)).await;
//         tx2.send(2).expect("channel should be open");
//     });

//     let now = Instant::now();
//     while let Some(x) = rx.recv().await {
//         println!("Received msg {x:?} after {dur:?}", dur = now.elapsed());
//     }
//     println!("Shutting down after {dur:?}", dur = now.elapsed());
// }
