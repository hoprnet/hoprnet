mod server_encode {
    use async_h1::server::Encoder;
    use async_std::io::Cursor;
    use async_std::io::ReadExt;
    use http_types::Body;
    use http_types::Result;
    use http_types::StatusCode;
    use http_types::{Method, Response};
    use pretty_assertions::assert_eq;

    async fn encode_to_string(
        response: Response,
        len: usize,
        method: Method,
    ) -> http_types::Result<String> {
        let mut buf = vec![];
        let mut encoder = Encoder::new(response, method);
        loop {
            let mut inner_buf = vec![0; len];
            let bytes = encoder.read(&mut inner_buf).await?;
            buf.extend_from_slice(&inner_buf[..bytes]);
            if bytes == 0 {
                return Ok(String::from_utf8(buf)?);
            }
        }
    }

    async fn assert_encoded(len: usize, method: Method, response: Response, lines: Vec<&str>) {
        assert_eq!(
            encode_to_string(response, len, method)
                .await
                .unwrap()
                .split("\r\n")
                .map(|line| {
                    if line.starts_with("date:") {
                        "date: {DATE}"
                    } else {
                        line
                    }
                })
                .collect::<Vec<_>>()
                .join("\r\n"),
            lines.join("\r\n")
        );
    }

    #[async_std::test]
    async fn basic() -> Result<()> {
        let res = Response::new(StatusCode::Ok);

        assert_encoded(
            100,
            Method::Get,
            res,
            vec![
                "HTTP/1.1 200 OK",
                "content-length: 0",
                "date: {DATE}",
                "",
                "",
            ],
        )
        .await;

        Ok(())
    }

    #[async_std::test]
    async fn basic_404() -> Result<()> {
        let res = Response::new(StatusCode::NotFound);

        assert_encoded(
            100,
            Method::Get,
            res,
            vec![
                "HTTP/1.1 404 Not Found",
                "content-length: 0",
                "date: {DATE}",
                "",
                "",
            ],
        )
        .await;

        Ok(())
    }

    #[async_std::test]
    async fn chunked() -> Result<()> {
        let mut res = Response::new(StatusCode::Ok);
        res.set_body(Body::from_reader(Cursor::new("hello world"), None));

        assert_encoded(
            10,
            Method::Get,
            res,
            vec![
                "HTTP/1.1 200 OK",
                "content-type: application/octet-stream",
                "date: {DATE}",
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
        Ok(())
    }

    #[async_std::test]
    async fn head_request_fixed_body() -> Result<()> {
        let mut res = Response::new(StatusCode::Ok);
        res.set_body("empty body because head request");

        assert_encoded(
            10,
            Method::Head,
            res,
            vec![
                "HTTP/1.1 200 OK",
                "content-length: 31",
                "content-type: text/plain;charset=utf-8",
                "date: {DATE}",
                "",
                "",
            ],
        )
        .await;

        Ok(())
    }

    #[async_std::test]
    async fn head_request_chunked_body() -> Result<()> {
        let mut res = Response::new(StatusCode::Ok);
        res.set_body(Body::from_reader(
            Cursor::new("empty body because head request"),
            None,
        ));

        assert_encoded(
            10,
            Method::Head,
            res,
            vec![
                "HTTP/1.1 200 OK",
                "content-type: application/octet-stream",
                "date: {DATE}",
                "transfer-encoding: chunked",
                "",
                "",
            ],
        )
        .await;

        Ok(())
    }
}
