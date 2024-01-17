use async_std::prelude::*;
use tide_websockets::{Message, WebSocket};

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    env_logger::init();
    let mut app = tide::new();
    app.at("/as_middleware")
        .with(WebSocket::new(|_request, mut stream| async move {
            while let Some(Ok(Message::Text(input))) = stream.next().await {
                let output: String = input.chars().rev().collect();

                stream
                    .send_string(format!("{} | {}", &input, &output))
                    .await?;
            }

            Ok(())
        }))
        .get(|_| async move { Ok("this was not a websocket request") });

    app.at("/as_endpoint")
        .get(WebSocket::new(|_request, mut stream| async move {
            while let Some(Ok(Message::Text(input))) = stream.next().await {
                let output: String = input.chars().rev().collect();

                stream
                    .send_string(format!("{} | {}", &input, &output))
                    .await?;
            }

            Ok(())
        }));

    app.listen("127.0.0.1:8080").await?;

    Ok(())
}
