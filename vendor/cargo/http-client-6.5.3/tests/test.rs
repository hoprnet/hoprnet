use mockito::mock;

use http_client::HttpClient;
use http_types::{Body, Request, Response, Url};

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(not(feature = "hyper_client"))] {
        use async_std::test as atest;
    } else {
        use tokio::test as atest;
    }
}

cfg_if! {
    if #[cfg(feature = "curl_client")] {
        use http_client::isahc::IsahcClient as DefaultClient;
    } else if #[cfg(feature = "wasm_client")] {
        use http_client::wasm::WasmClient as DefaultClient;
    } else if #[cfg(any(feature = "h1_client", feature = "h1_client_rustls"))] {
        use http_client::h1::H1Client as DefaultClient;
    } else if #[cfg(feature = "hyper_client")] {
        use http_client::hyper::HyperClient as DefaultClient;
    }
}

#[atest]
async fn post_json() -> Result<(), http_types::Error> {
    #[derive(serde::Deserialize, serde::Serialize)]
    struct Cat {
        name: String,
    }

    let cat = Cat {
        name: "Chashu".to_string(),
    };

    let m = mock("POST", "/")
        .with_status(200)
        .match_body(&serde_json::to_string(&cat)?[..])
        .with_body(&serde_json::to_string(&cat)?[..])
        .create();
    let mut req = Request::new(
        http_types::Method::Post,
        Url::parse(&mockito::server_url()).unwrap(),
    );
    req.append_header("Accept", "application/json");
    req.set_body(Body::from_json(&cat)?);
    let res: Response = DefaultClient::new().send(req).await?;
    m.assert();
    assert_eq!(res.status(), http_types::StatusCode::Ok);
    Ok(())
}

#[atest]
async fn get_json() -> Result<(), http_types::Error> {
    #[derive(serde::Deserialize)]
    struct Message {
        message: String,
    }
    let m = mock("GET", "/")
        .with_status(200)
        .with_body(r#"{"message": "hello, world!"}"#)
        .create();
    let req = Request::new(
        http_types::Method::Get,
        Url::parse(&mockito::server_url()).unwrap(),
    );
    let mut res: Response = DefaultClient::new().send(req).await?;
    let msg: Message = serde_json::from_str(&res.body_string().await?)?;
    m.assert();
    assert_eq!(msg.message, "hello, world!");
    Ok(())
}

#[atest]
async fn get_google() -> Result<(), http_types::Error> {
    let url = "https://www.google.com";
    let req = Request::new(http_types::Method::Get, Url::parse(url).unwrap());
    let mut res: Response = DefaultClient::new().send(req).await?;
    assert_eq!(res.status(), http_types::StatusCode::Ok);

    let msg = res.body_bytes().await?;
    let msg = String::from_utf8_lossy(&msg);
    println!("recieved: '{}'", msg);
    assert!(msg.contains("<!doctype html>"));
    assert!(msg.contains("<title>Google</title>"));
    assert!(msg.contains("<head>"));
    assert!(msg.contains("</head>"));
    assert!(msg.contains("</script>"));
    assert!(msg.contains("</script>"));

    assert!(msg.contains("<body"));
    assert!(msg.contains("</body>"));
    assert!(msg.contains("</html>"));

    Ok(())
}

#[atest]
async fn get_github() -> Result<(), http_types::Error> {
    let url = "https://raw.githubusercontent.com/http-rs/surf/6627d9fc15437aea3c0a69e0b620ae7769ea6765/LICENSE-MIT";
    let req = Request::new(http_types::Method::Get, Url::parse(url).unwrap());
    let mut res: Response = DefaultClient::new().send(req).await?;
    assert_eq!(res.status(), http_types::StatusCode::Ok, "{:?}", &res);

    let msg = res.body_string().await?;

    assert_eq!(
        msg,
        "The MIT License (MIT)

Copyright (c) 2019 Yoshua Wuyts

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the \"Software\"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED \"AS IS\", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
"
    );

    Ok(())
}

#[atest]
async fn keep_alive() {
    let _mock_guard = mockito::mock("GET", "/report")
        .with_status(200)
        .expect_at_least(2)
        .create();

    let client = DefaultClient::new();
    let url: Url = format!("{}/report", mockito::server_url()).parse().unwrap();
    let req = Request::new(http_types::Method::Get, url);
    client.send(req.clone()).await.unwrap();
    client.send(req.clone()).await.unwrap();
}

#[atest]
async fn fallback_to_ipv4() {
    let client = DefaultClient::new();
    let _mock_guard = mock("GET", "/")
        .with_status(200)
        .expect_at_least(2)
        .create();

    // Kips the initial "http://127.0.0.1:" to get only the port number
    let mock_port = &mockito::server_url()[17..];

    let url = &format!("http://localhost:{}", mock_port);
    let req = Request::new(http_types::Method::Get, Url::parse(url).unwrap());
    client.send(req.clone()).await.unwrap();
}
