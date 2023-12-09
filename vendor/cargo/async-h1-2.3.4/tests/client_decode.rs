mod test_utils;

mod client_decode {
    use std::io::Write;

    use super::test_utils::CloseableCursor;
    use async_h1::client;
    use async_std::io::Cursor;
    use http_types::headers;
    use http_types::Response;
    use http_types::Result;
    use pretty_assertions::assert_eq;

    async fn decode_lines(s: Vec<&str>) -> Result<Response> {
        client::decode(Cursor::new(s.join("\r\n"))).await
    }

    #[async_std::test]
    async fn response_no_date() -> Result<()> {
        let res = decode_lines(vec![
            "HTTP/1.1 200 OK",
            "transfer-encoding: chunked",
            "content-type: text/plain",
            "",
            "",
        ])
        .await?;

        assert!(res.header(&headers::DATE).is_some());
        Ok(())
    }

    #[async_std::test]
    async fn multiple_header_values_for_same_header_name() -> Result<()> {
        let res = decode_lines(vec![
            "HTTP/1.1 200 OK",
            "host: example.com",
            "content-length: 0",
            "set-cookie: sessionId=e8bb43229de9",
            "set-cookie: qwerty=219ffwef9w0f",
            "",
            "",
        ])
        .await?;
        assert_eq!(res.header(&headers::SET_COOKIE).unwrap().iter().count(), 2);

        Ok(())
    }

    #[async_std::test]
    async fn connection_closure() -> Result<()> {
        let mut cursor = CloseableCursor::default();
        cursor.write_all(b"HTTP/1.1 200 OK\r\nhost: example.com")?;
        cursor.close();
        assert_eq!(
            client::decode(cursor).await.unwrap_err().to_string(),
            "empty response"
        );

        let cursor = CloseableCursor::default();
        cursor.close();
        assert_eq!(
            client::decode(cursor).await.unwrap_err().to_string(),
            "connection closed"
        );

        Ok(())
    }

    #[async_std::test]
    async fn response_newlines() -> Result<()> {
        let res = decode_lines(vec![
            "HTTP/1.1 200 OK",
            "content-length: 78",
            "date: {DATE}",
            "content-type: text/plain; charset=utf-8",
            "",
            "http specifies headers are separated with \r\n but many servers don't do that",
            "",
        ])
        .await?;

        assert_eq!(
            res[headers::CONTENT_LENGTH]
                .as_str()
                .parse::<usize>()
                .unwrap(),
            78
        );

        Ok(())
    }
}
