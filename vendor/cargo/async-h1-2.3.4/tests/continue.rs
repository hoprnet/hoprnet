mod test_utils;

use async_std::{io, prelude::*, task};
use http_types::Result;
use std::time::Duration;
use test_utils::TestIO;

const REQUEST_WITH_EXPECT: &[u8] = b"POST / HTTP/1.1\r\n\
Host: example.com\r\n\
Content-Length: 10\r\n\
Expect: 100-continue\r\n\r\n";

const SLEEP_DURATION: Duration = std::time::Duration::from_millis(100);
#[async_std::test]
async fn test_with_expect_when_reading_body() -> Result<()> {
    let (mut client, server) = TestIO::new();
    client.write_all(REQUEST_WITH_EXPECT).await?;

    let (mut request, _) = async_h1::server::decode(server).await?.unwrap();

    task::sleep(SLEEP_DURATION).await; //prove we're not just testing before we've written

    assert_eq!("", &client.read.to_string()); // we haven't written yet

    let join_handle = task::spawn(async move {
        let mut string = String::new();
        request.read_to_string(&mut string).await?; //this triggers the 100-continue even though there's nothing to read yet
        io::Result::Ok(string)
    });

    task::sleep(SLEEP_DURATION).await; // just long enough to wait for the channel and io

    assert_eq!("HTTP/1.1 100 Continue\r\n\r\n", &client.read.to_string());

    client.write_all(b"0123456789").await?;

    assert_eq!("0123456789", &join_handle.await?);

    Ok(())
}

#[async_std::test]
async fn test_without_expect_when_not_reading_body() -> Result<()> {
    let (mut client, server) = TestIO::new();
    client.write_all(REQUEST_WITH_EXPECT).await?;

    let (_, _) = async_h1::server::decode(server).await?.unwrap();

    task::sleep(SLEEP_DURATION).await; // just long enough to wait for the channel

    assert_eq!("", &client.read.to_string()); // we haven't written 100-continue

    Ok(())
}
