# Wallets

At HOPR we create and use multiple Ethereum addresses that are under the control and supervision of the HOPR Association.

Here are some of these wallets:

- `0xA18732DC751BE0dB04157eb92C92BA9d0fC09FC5` (codename: Deployer)
- `0x1A387b5103f28bc6601d085A3dDC878dEE631A56` (codename: Minter)
- `0x34465FE0B25089Fc9D3a6D33e19F652e45B175e0` (codename: Alice)
- `0x2402da10A6172ED018AEEa22CA60EDe1F766655C` (codename: Dev Bank)
- `0x7dB59a3c1e8505845F4a8BF373fD2Cff42037eBd` (codename: Dev External)

There are some additional wallets used for testing, that had been label `[ Unknown ]`. They will be handled in https://github.com/hoprnet/hoprnet/issues/2893.

# Policy

HOPR Association [multi-sig](https://etherscan.io/address/0x4f50ab4e931289344a57f2fe4bbd10546a6fdc17) is the main address where all HOPR related funds are stored and controlled.

All other wallets defined in [wallets](#Wallets) are or had been in control of Association contributors or HOPR Services AG employees. As they are controlled via private keys, mainnet funds are usually short-live and communicated internally.

All assets, current or future, that exist in [wallets](#Wallets), independently of the blockchain they live on, should be considered Association property and can only be used for development purposes. When their purpose is complete, they should be sent back to the Association.

Additional wallets that are not defined under the [wallets](#Wallets) have no connections to the Association whatsoever, and are the sole responsibility of their owners, independently of their relationship with the Association.

No HD-derived wallets (e.g. mnemonics) are used for HOPR Association as having the seed of this wallet would grant access to private keys that could be used further down the line w/o being aware of that being the case.

# Generation

There are multiple ways to safely create wallets, but for quick and dirty (disposable) wallets which require not much scrutiny, feel free to use the following JS script runnable in a web console:

```js
;((_) => _.reduce((a, v) => `${v.toString(16).padStart(2, '0')}${a}`, ''))(
  ((_) => crypto.getRandomValues(_))(new Uint8Array(32))
)
```
