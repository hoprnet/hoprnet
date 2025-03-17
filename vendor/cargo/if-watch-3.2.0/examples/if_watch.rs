use futures::StreamExt;
use if_watch::smol::IfWatcher;

fn main() {
    env_logger::init();
    smol::block_on(async {
        let mut set = IfWatcher::new().unwrap();
        loop {
            let event = set.select_next_some().await;
            println!("Got event {:?}", event);
        }
    });
}
