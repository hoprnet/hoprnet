# hoprd-keypair

Generates, reads, and writes hopr key material following the Ethereum KeyStore standard.

Reimplements the `eth_keystore` crate to support WASM which cannot be used because using custom FS access is not foreseen and utilized version of `uuid` cannot use JS `getrandom` entropy.
