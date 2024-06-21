#![cfg(not(miri))]
use std::time::Instant;

use futures_buffered::BufferedStreamExt;
use futures_util::StreamExt;
use reqwest::{Client, Error};

static URLS: &[&str] = &[
    "https://api.ipify.org/",
    "https://www.boredapi.com/api/activity",
    "https://random.dog/woof.json",
];

#[tokio::test]
async fn futures_util() -> Result<(), Error> {
    let http = Client::new();

    let start = Instant::now();

    futures::stream::iter(URLS)
        .cycle()
        .take(256)
        .map(|&url| {
            let client = &http;
            async move {
                let resp = client.get(url).send().await?;
                let status = resp.status();
                let text = resp.text().await;
                Ok::<_, Error>((url, status, text))
            }
        })
        .buffer_unordered(8)
        .for_each(|res| async {
            if let Ok((url, status, Ok(text))) = res {
                println!("{url} ({status}) {text}");
            }
        })
        .await;

    println!("end {:?}", start.elapsed());

    Ok(())
}

#[tokio::test]
async fn futures_buffered() -> Result<(), Error> {
    let http = Client::new();

    let start = Instant::now();

    futures::stream::iter(URLS)
        .cycle()
        .take(256)
        .map(|&url| {
            let client = &http;
            async move {
                let resp = client.get(url).send().await?;
                let status = resp.status();
                let text = resp.text().await;
                Ok::<_, Error>((url, status, text))
            }
        })
        .buffered_unordered(8)
        .for_each(|res| async {
            if let Ok((url, status, Ok(text))) = res {
                println!("{url} ({status}) {text}");
            }
        })
        .await;

    println!("end {:?}", start.elapsed());

    Ok(())
}
