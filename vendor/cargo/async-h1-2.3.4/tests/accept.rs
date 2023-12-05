mod test_utils;
mod accept {
    use super::test_utils::TestServer;
    use async_h1::{client::Encoder, server::ConnectionStatus};
    use async_std::io::{self, prelude::WriteExt, Cursor};
    use http_types::{headers::CONNECTION, Body, Request, Response, Result};

    #[async_std::test]
    async fn basic() -> Result<()> {
        let mut server = TestServer::new(|req| async {
            let mut response = Response::new(200);
            let len = req.len();
            response.set_body(Body::from_reader(req, len));
            Ok(response)
        });

        let content_length = 10;

        let request_str = format!(
            "POST / HTTP/1.1\r\nHost: example.com\r\nContent-Length: {}\r\n\r\n{}\r\n\r\n",
            content_length,
            std::str::from_utf8(&vec![b'|'; content_length]).unwrap()
        );

        server.write_all(request_str.as_bytes()).await?;
        assert_eq!(server.accept_one().await?, ConnectionStatus::KeepAlive);

        server.close();
        assert_eq!(server.accept_one().await?, ConnectionStatus::Close);

        assert!(server.all_read());

        Ok(())
    }

    #[async_std::test]
    async fn request_close() -> Result<()> {
        let mut server = TestServer::new(|_| async { Ok(Response::new(200)) });

        server
            .write_all(b"GET / HTTP/1.1\r\nHost: example.com\r\nConnection: Close\r\n\r\n")
            .await?;

        assert_eq!(server.accept_one().await?, ConnectionStatus::Close);

        assert!(server.all_read());

        Ok(())
    }

    #[async_std::test]
    async fn response_close() -> Result<()> {
        let mut server = TestServer::new(|_| async {
            let mut response = Response::new(200);
            response.insert_header(CONNECTION, "close");
            Ok(response)
        });

        server
            .write_all(b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n")
            .await?;

        assert_eq!(server.accept_one().await?, ConnectionStatus::Close);

        assert!(server.all_read());

        Ok(())
    }

    #[async_std::test]
    async fn keep_alive_short_fixed_length_unread_body() -> Result<()> {
        let mut server = TestServer::new(|_| async { Ok(Response::new(200)) });

        let content_length = 10;

        let request_str = format!(
            "POST / HTTP/1.1\r\nHost: example.com\r\nContent-Length: {}\r\n\r\n{}\r\n\r\n",
            content_length,
            std::str::from_utf8(&vec![b'|'; content_length]).unwrap()
        );

        server.write_all(request_str.as_bytes()).await?;
        assert_eq!(server.accept_one().await?, ConnectionStatus::KeepAlive);

        server.write_all(request_str.as_bytes()).await?;
        assert_eq!(server.accept_one().await?, ConnectionStatus::KeepAlive);

        server.close();
        assert_eq!(server.accept_one().await?, ConnectionStatus::Close);

        assert!(server.all_read());

        Ok(())
    }

    #[async_std::test]
    async fn keep_alive_short_chunked_unread_body() -> Result<()> {
        let mut server = TestServer::new(|_| async { Ok(Response::new(200)) });

        let content_length = 100;

        let mut request = Request::post("http://example.com/");
        request.set_body(Body::from_reader(
            Cursor::new(vec![b'|'; content_length]),
            None,
        ));

        io::copy(&mut Encoder::new(request), &mut server).await?;
        assert_eq!(server.accept_one().await?, ConnectionStatus::KeepAlive);

        server
            .write_fmt(format_args!(
                "GET / HTTP/1.1\r\nHost: example.com\r\nContent-Length: 0\r\n\r\n"
            ))
            .await?;
        assert_eq!(server.accept_one().await?, ConnectionStatus::KeepAlive);

        server.close();
        assert_eq!(server.accept_one().await?, ConnectionStatus::Close);

        assert!(server.all_read());

        Ok(())
    }

    #[async_std::test]
    async fn keep_alive_long_fixed_length_unread_body() -> Result<()> {
        let mut server = TestServer::new(|_| async { Ok(Response::new(200)) });

        let content_length = 10000;

        let request_str = format!(
            "POST / HTTP/1.1\r\nHost: example.com\r\nContent-Length: {}\r\n\r\n{}\r\n\r\n",
            content_length,
            std::str::from_utf8(&vec![b'|'; content_length]).unwrap()
        );

        server.write_all(request_str.as_bytes()).await?;
        assert_eq!(server.accept_one().await?, ConnectionStatus::KeepAlive);

        server.write_all(request_str.as_bytes()).await?;
        assert_eq!(server.accept_one().await?, ConnectionStatus::KeepAlive);

        server.close();
        assert_eq!(server.accept_one().await?, ConnectionStatus::Close);

        assert!(server.all_read());

        Ok(())
    }

    #[async_std::test]
    async fn keep_alive_long_chunked_unread_body() -> Result<()> {
        let mut server = TestServer::new(|_| async { Ok(Response::new(200)) });

        let content_length = 10000;

        let mut request = Request::post("http://example.com/");
        request.set_body(Body::from_reader(
            Cursor::new(vec![b'|'; content_length]),
            None,
        ));

        server.write_request(request).await?;
        assert_eq!(server.accept_one().await?, ConnectionStatus::KeepAlive);

        server
            .write_fmt(format_args!(
                "GET / HTTP/1.1\r\nHost: example.com\r\nContent-Length: 0\r\n\r\n"
            ))
            .await?;
        assert_eq!(server.accept_one().await?, ConnectionStatus::KeepAlive);

        server.close();
        assert_eq!(server.accept_one().await?, ConnectionStatus::Close);

        assert!(server.all_read());

        Ok(())
    }
}
