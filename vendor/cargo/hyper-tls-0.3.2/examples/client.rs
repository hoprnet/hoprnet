extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate tokio;

use futures::{future, Future, Stream};
use std::io::Write;

fn main() {
    tokio::run(future::lazy(|| {
        let https = hyper_tls::HttpsConnector::new(4).unwrap();
        let client = hyper::Client::builder()
            .build::<_, hyper::Body>(https);

        client
            .get("https://hyper.rs".parse().unwrap())
            .and_then(|res| {
                println!("Status: {}", res.status());
                println!("Headers:\n{:#?}", res.headers());
                res.into_body().for_each(|chunk| {
                    ::std::io::stdout()
                        .write_all(&chunk)
                        .map_err(|e| panic!("example expects stdout to work: {}", e))
                })
            })
            .map_err(|e| println!("request error: {}", e))
    }));
}
