# Wallets

At HOPR we create and use multiple Ethereum addresses that are under the control and supervision of the HOPR Association.

Here are some of these wallets:

| Wallet Address                               | Alias                              | Description                                                                       |
| -------------------------------------------- | ---------------------------------- | --------------------------------------------------------------------------------- |
| `0xA18732DC751BE0dB04157eb92C92BA9d0fC09FC5` | Dev Deployer                       | Used for deploying testnet or demo contracts across our network.                  |
| `0x1A387b5103f28bc6601d085A3dDC878dEE631A56` | Dev Minter                         | Used as user granted `mint`ing access to our test contracts in our network.       |
| `0x34465FE0B25089Fc9D3a6D33e19F652e45B175e0` | Alice                              | Used as first user for interacting with some of our contracts (e.g. `HOPRBoost`)  |
| `0x2402da10A6172ED018AEEa22CA60EDe1F766655C` | Dev Bank                           | `all-hands` wallet used by all HOPR team members to fund nodes or other accounts. |
| `0x7dB59a3c1e8505845F4a8BF373fD2Cff42037eBd` | Dev External                       | limited access wallet granted to external/community members                       |
| `0xD7682Ef1180f5Fc496CF6981e4854738a57c593E` | NFT Minter                         | Used for minting NFTs when `mint`ing role is given. It renounces it shortly after |
| `0x4f50ab4e931289344a57f2fe4bbd10546a6fdc17` | HOPR Association Gnosis Wallet     | Used for paying services to all different parties involved with HOPR Association  |
| `0x5E1c4e7004B7411bA27Dc354330fab31147DFeF1` | HOPR Asociation xDAI Gnosis Wallet | Same as “HOPR Association Gnosis Wallet” wallet but on the xDAI network           |
| `0x752af2bf9dbbc1105a83d2ca1ee8f1046d85b702` | HOPR Association Gnosis Safe       | Same as “HOPR Association Gnosis Wallet” wallet but using the new Gnosis Safe     |
| `0xE9131488563776DE7FEa238d6112c5dA46be9a9F` | HOPR Association xDAI Gnosis Safe  | Same as “HOPR Association Gnosis Safe” wallet but on the xDAI network             |
| `0x2D8E358487FeDa42629274CE041F98629Bf65cF3` | HOPR DAO main net Gnosis Safe      | Used to ratify actions on behalf of HOPR's DAO and holding Uniswap Liquidity fees |
| `0x8f7a2AbbC8741572427e3426538cD516A41102f3` | HOPR Deployer                      | Main net minter account & HOPR Association and DAO multisig representative        |
| `0x5AB4f2a41DEb3B925B23a3f7E00F206BED18ABB3` | Multisig #1                        | HOPR Association representative #1 (both Gnosis Wallet + Gnosis Safe              |
| `0x93bC372b4cC142dA75a365C5cB45be996347bfeC` | Multisig #2                        | HOPR Association representative #2 (only Gnosis Safe)                             |
| `0x50677B7e720102c5126e17f4485149208d3fce71` | Multisig #3                        | HOPR Association representative #3 (only Gnosis Wallet)                           |

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
