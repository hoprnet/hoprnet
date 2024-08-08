# surf-governor

A rate-limiting middleware for surf

## Install

With [cargo add](https://github.com/killercup/cargo-edit#Installation) installed :

```sh
cargo add surf-governor
```

## Documentation

- [API Docs](https://docs.rs/surf-governor)

## Example

 ```rust
 use surf_governor::GovernorMiddleware;
 use surf::{Client, Request, http::Method};
 use url::Url;

 #[async_std::main]
 async fn main() -> surf::Result<()> {
     let req = Request::new(Method::Get, Url::parse("https://example.api")?);
     let client = Client::new().with(GovernorMiddleware::per_second(1)?);
     let res = client.send(req).await?;
     Ok(())
 }
 ```
