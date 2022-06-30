# Tokens Process

The purpose of this process is to document how the deployed tokens and their
usage.

## Tokens

| Address                                                    | Network          | Name         | Symbol | Description                                                          |
| ---------------------------------------------------------- | ---------------- | ------------ | ------ | -------------------------------------------------------------------- |
| [`0xf5581dfefd8fb0e4aec526be659cfab1f8c781da`][es_hopr]    | Ethereum Mainnet | Hopr         | Hopr   | Main token.                                                          |
| [`0xD057604A14982FE8D88c5fC25Aac3267eA142a08`][bs_xhopr]   | Gnosis Chain     | Hopr         | xHopr  | Bridged token from Ethereum Mainnet.                                 |
| [`0xD4fdec44DB9D44B8f2b6d529620f9C0C7066A2c1`][bs_wxhopr]  | Gnosis Chain     | Wrapped Hopr | wxHopr | ERC777 version of xHopr. E.g. used in staking rewards.               |
| [`0xe8aD2ac347dA7549Aaca8F5B1c5Bf979d85bC78F`][bs_txhopr]  | Gnosis Chain     | Test Hopr    | txHopr | Used for integration testing by various systems. Minted as required. |
| [`0xa3C8f4044b30Fb3071F5b3b02913DE524F1041dc`][esg_txhopr] | Ethereum Goerli  | Test Hopr    | txHopr | Used for integration testing by various systems. Minted as required. |

[es_hopr]: https://etherscan.io/token/0xf5581dfefd8fb0e4aec526be659cfab1f8c781da
[bs_txhopr]: https://blockscout.com/xdai/mainnet/address/0xe8aD2ac347dA7549Aaca8F5B1c5Bf979d85bC78F
[bs_wxhopr]: https://blockscout.com/xdai/mainnet/address/0xD4fdec44DB9D44B8f2b6d529620f9C0C7066A2c1
[bs_xhopr]: https://blockscout.com/xdai/mainnet/address/0xD057604A14982FE8D88c5fC25Aac3267eA142a08
[esg_txhopr]: https://goerli.etherscan.io/token/0xa3C8f4044b30Fb3071F5b3b02913DE524F1041dc
