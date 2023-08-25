# Cross platform asynchronous network watcher

```sh
cargo run --example if_watch
Got event Ok(Up(127.0.0.1/8))
Got event Ok(Up(192.168.6.65/24))
Got event Ok(Up(::1/128))
Got event Ok(Up(2a01:8b81:7000:9700:cef9:e4ff:fe9e:b23b/64))
Got event Ok(Up(fe80::cef9:e4ff:fe9e:b23b/64))
```

Supported platforms at the moment are:
Linux, Windows and Android with a fallback for Macos and ios that polls for changes every 10s.

## License
MIT OR Apache-2.0
