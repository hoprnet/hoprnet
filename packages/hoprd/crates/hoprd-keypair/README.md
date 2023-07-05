# hoprd-keypair

Generates, reads, and writes hopr key material following the Ethereum KeyStore standard.

Reimplements the `eth_keystore` crate to support WASM which cannot be used because using custom FS access is not foreseen and utilized version of `uuid` cannot use JS `getrandom` entropy.

Automatically migrates keyStore that only have a `chain_key` to those who also store a `packet_key`.

# Key format

```json
{
  "version": 2,
  "chain_key": "7a714c922cd769ab6702ffac411fb1a4d28500613e2ac8bf71f14c3604bba091",
  "packet_key": "5d147b58bce1664f88e60921a9aa8195cc54081e587b24e66005bbf60fbf480c"
}
```

# Example

Key store file (password: `e2e-test`)

```json
{
  "crypto": {
    "cipher": "aes-128-ctr",
    "cipherparams": { "iv": "3b0551f925bfd0ded154ec487dc78d29" },
    "ciphertext": "bbdb25cdbeff683926999baee0e929c4e9922ca0b4c99e90351aa8fa286b10d7a5bcdfe2e58cffc9d4f9df10121d0b4b1ac697f97909e8f9fdf15aff91b555cb7ca8b5e10ed747de9a99c0e9d3c540ed09997689ec5aba0d0a946e5ea167c7e4be91fa67be419fd0169aca1d73229d0049bff82e3f6c3256c7a2ba24bb4b02aefa224f0ad70479a5e117b3cc133adf03021aceb2152b8727ccd559ce3758bf523e429a4f4806b58fa5597532",
    "kdf": "scrypt",
    "kdfparams": {
      "dklen": 32,
      "n": 2,
      "p": 1,
      "r": 8,
      "salt": "01da6de6a096ba594fd1119ad907e8fbf531874a4bcc234a3a88b2e9e4d8cb06"
    },
    "mac": "a18ae65efbc0d0085ff6a26956695f704509cdb6a05ee168e2017c4399e8be43"
  },
  "id": "68172b5b-e4e7-4ac9-9932-352a95f11561",
  "version": 3
}
```

leading to following PeerIds:

```
chain_key 16Uiu2HAmJqfGeZPa8VJ8263NDjehHkMXYqYzbi4zqhH7Y3RKsEoV (secp256k1)
packet_key 12D3KooWPGsW7vZ8VsmJ9Lws9vsKaBiACZXQ3omRm3rFUho5BpvF (ed25519)

```
