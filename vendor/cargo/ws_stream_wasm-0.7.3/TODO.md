# TODO

- remove this_error...
- tests for new behavior of WsMessage tryfrom and error handling.
- doc tests
- look into proper changelogs, like the futures crate.
- update tokio-util, breaking change, probably needs new version of tokio-serde-cbor
- ci: https://rustwasm.github.io/docs/wasm-bindgen/wasm-bindgen-test/continuous-integration.html has some windows instructions.

## Features
- when the connection is lost, can we know if it's the server that disconnected (correct shutdown exchange)
  or whether we have network problems.
- reconnect?

## Testing

## Documentation
- chat client example
- automatic reconnect example using pharos to detect the close



