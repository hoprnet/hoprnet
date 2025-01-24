#[cfg(feature = "aio_tokio")]
use hyper::{
    header::{CONTENT_LENGTH, CONTENT_TYPE},
    Body, Client, Request,
};

use crate::errors::RequestError;

#[derive(Clone, Debug)]
pub struct Action(String);

impl Action {
    pub fn new(action: &str) -> Action {
        Action(action.into())
    }
}

const HEADER_NAME: &str = "SOAPAction";

#[cfg(feature = "aio_async_std")]
pub async fn send_async(url: &str, action: Action, body: &str) -> Result<String, RequestError> {
    Ok(surf::post(url)
        .header(HEADER_NAME, action.0)
        .content_type("text/xml")
        .body(body)
        .recv_string()
        .await?)
}

#[cfg(feature = "aio_tokio")]
pub async fn send_async(url: &str, action: Action, body: &str) -> Result<String, RequestError> {
    let client = Client::new();

    let req = Request::builder()
        .uri(url)
        .method("POST")
        .header(HEADER_NAME, action.0)
        .header(CONTENT_TYPE, "text/xml")
        .header(CONTENT_LENGTH, body.len() as u64)
        .body(Body::from(body.to_string()))?;

    let resp = client.request(req).await?;
    let body = hyper::body::to_bytes(resp.into_body()).await?;
    let string = String::from_utf8(body.to_vec())?;
    Ok(string)
}
