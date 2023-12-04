use http_client::HttpClient;
use http_types::{Method, Request};

#[cfg(any(feature = "h1_client", feature = "docs"))]
use http_client::h1::H1Client as Client;
#[cfg(all(feature = "hyper_client", not(feature = "docs")))]
use http_client::hyper::HyperClient as Client;
#[cfg(all(feature = "curl_client", not(feature = "docs")))]
use http_client::isahc::IsahcClient as Client;
#[cfg(all(feature = "wasm_client", not(feature = "docs")))]
use http_client::wasm::WasmClient as Client;

#[async_std::main]
async fn main() {
    let client = Client::new();

    let req = Request::new(Method::Get, "http://example.org");

    client.send(req).await.unwrap();

    dbg!(client);
}
