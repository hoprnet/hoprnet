# hoprd-keypair

Generates, reads, and writes hopr key material following the Ethereum KeyStore standard.

Reimplements the `eth_keystore` crate to support WASM which cannot be used because using custom FS access is not foreseen and utilized version of `uuid` cannot use JS `getrandom` entropy.

Automatically migrates keyStore that only have a `chain_key` to those who also store a `packet_key`

# Key format

```
32 bytes packet key (binary)
32 bytes chain key (binary)
```

# Example

Key store file (password: `e2e-test`)

```json
{
  "crypto": {
    "cipher": "aes-128-ctr",
    "cipherparams": { "iv": "7ac739b4195235115a2faf0b0da82c27" },
    "ciphertext": "d27ff5c553ece727f5385238fe9bc356a6cda7f2c8bc5b4634ef1e2de7f909d7d3a53fb0cee659e508d09fe30765774d6115d1f7770dba86ff43a0ef0e5635ab",
    "kdf": "scrypt",
    "kdfparams": {
      "dklen": 32,
      "n": 2,
      "p": 1,
      "r": 8,
      "salt": "be718b4ca2e661baaec57774109bbe6a24b1fa8d677cf5e02b2e9e884301a1fd"
    },
    "mac": "bcea10e039915e54171a17c0f6e696bd76c3135d1fbdddd18055fc8859ff7fff"
  },
  "id": "8af79886-dc4f-483f-9268-d467882d3ff3",
  "version": 3
}
```

leading to following PeerIds:

```
chain_key 16Uiu2HAmUYnGY3USo8iy13SBFW7m5BMQvC4NETu1fGTdoB86piw7 (secp256k1)
packet_key 12D3KooWRiX9oihUK1n8n4W9N5B1dFpkQFKNrDeYuAytRsYamTnN (ed25519)

```
