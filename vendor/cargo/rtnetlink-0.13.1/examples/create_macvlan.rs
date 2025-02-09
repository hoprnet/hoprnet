// SPDX-License-Identifier: MIT

use futures::stream::TryStreamExt;
use rtnetlink::{new_connection, Error, Handle};
use std::env;

#[tokio::main]
async fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        usage();
        return Ok(());
    }
    let link_name = &args[1];

    let (connection, handle, _) = new_connection().unwrap();
    tokio::spawn(connection);

    create_macvlan(handle, link_name.to_string())
        .await
        .map_err(|e| format!("{e}"))
}

async fn create_macvlan(
    handle: Handle,
    link_name: String,
) -> Result<(), Error> {
    let mut links = handle.link().get().match_name(link_name.clone()).execute();
    if let Some(link) = links.try_next().await? {
        let request = handle.link().add().macvlan(
            "test_macvlan".into(),
            link.header.index,
            4u32, // bridge mode
        );
        request.execute().await?
    } else {
        println!("no link {link_name} found");
    }
    Ok(())
}

fn usage() {
    eprintln!(
        "usage:
    cargo run --example create_macvlan -- <link name>

Note that you need to run this program as root. Instead of running cargo as root,
build the example normally:

    cargo build --example create_macvlan

Then find the binary in the target directory:

    cd target/debug/examples ; sudo ./create_macvlan <link_name>"
    );
}
