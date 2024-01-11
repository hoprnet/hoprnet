mod client_encode {
    use async_h1::client;
    use async_std::io::Cursor;
    use async_std::prelude::*;
    use client::Encoder;
    use http_types::Body;
    use http_types::Result;
    use http_types::{Method, Request, Url};
    use pretty_assertions::assert_eq;

    async fn encode_to_string(request: Request, len: usize) -> http_types::Result<String> {
        let mut buf = vec![];
        let mut encoder = Encoder::new(request);
        loop {
            let mut inner_buf = vec![0; len];
            let bytes = encoder.read(&mut inner_buf).await?;
            buf.extend_from_slice(&inner_buf[..bytes]);
            if bytes == 0 {
                return Ok(String::from_utf8(buf)?);
            }
        }
    }

    #[async_std::test]
    async fn client_encode_request_add_date() -> Result<()> {
        let url = Url::parse("http://localhost:8080").unwrap();
        let mut req = Request::new(Method::Post, url);
        req.set_body("hello");

        assert_encoded(
            100,
            req,
            vec![
                "POST / HTTP/1.1",
                "host: localhost:8080",
                "content-length: 5",
                "content-type: text/plain;charset=utf-8",
                "",
                "hello",
            ],
        )
        .await;
        Ok(())
    }

    #[async_std::test]
    async fn client_encode_request_with_connect() -> Result<()> {
        let url = Url::parse("https://example.com:443").unwrap();
        let req = Request::new(Method::Connect, url);

        assert_encoded(
            100,
            req,
            vec![
                "CONNECT example.com:443 HTTP/1.1",
                "host: example.com",
                "content-length: 0",
                "proxy-connection: keep-alive",
                "",
                "",
            ],
        )
        .await;

        Ok(())
    }

    // The fragment of an URL is not send to the server, see RFC7230 and RFC3986.
    #[async_std::test]
    async fn client_encode_request_with_fragment() -> Result<()> {
        let url = Url::parse("http://example.com/path?query#fragment").unwrap();
        let req = Request::new(Method::Get, url);

        assert_encoded(
            10,
            req,
            vec![
                "GET /path?query HTTP/1.1",
                "host: example.com",
                "content-length: 0",
                "",
                "",
            ],
        )
        .await;

        Ok(())
    }

    async fn assert_encoded(len: usize, req: Request, lines: Vec<&str>) {
        assert_eq!(
            encode_to_string(req, len).await.unwrap(),
            lines.join("\r\n"),
        )
    }

    #[async_std::test]
    async fn client_encode_chunked_body() -> Result<()> {
        let url = Url::parse("http://example.com/path?query").unwrap();
        let mut req = Request::new(Method::Get, url.clone());
        req.set_body(Body::from_reader(Cursor::new("hello world"), None));

        assert_encoded(
            10,
            req,
            vec![
                "GET /path?query HTTP/1.1",
                "host: example.com",
                "content-type: application/octet-stream",
                "transfer-encoding: chunked",
                "",
                "5",
                "hello",
                "5",
                " worl",
                "1",
                "d",
                "0",
                "",
                "",
            ],
        )
        .await;

        let mut req = Request::new(Method::Get, url.clone());
        req.set_body(Body::from_reader(Cursor::new("hello world"), None));

        assert_encoded(
            16,
            req,
            vec![
                "GET /path?query HTTP/1.1",
                "host: example.com",
                "content-type: application/octet-stream",
                "transfer-encoding: chunked",
                "",
                "B",
                "hello world",
                "0",
                "",
                "",
            ],
        )
        .await;

        let mut req = Request::new(Method::Get, url.clone());
        req.set_body(Body::from_reader(
            Cursor::new(
                "this response is more than 32 bytes long in order to require a second hex digit",
            ),
            None,
        ));

        assert_encoded(
            32,
            req,
            vec![
                "GET /path?query HTTP/1.1",
                "host: example.com",
                "content-type: application/octet-stream",
                "transfer-encoding: chunked",
                "",
                "1A",
                "this response is more than",
                "1A",
                " 32 bytes long in order to",
                "1A",
                " require a second hex digi",
                "1",
                "t",
                "0",
                "",
                "",
            ],
        )
        .await;

        Ok(())
    }
}
