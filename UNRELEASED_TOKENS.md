# Guide

Read [#2093](https://github.com/hoprnet/hoprnet/issues/2093) for context and [#2102](https://github.com/hoprnet/hoprnet/issues/2102) for specifications.

## Words

| word            | description                                                                |
| --------------- | -------------------------------------------------------------------------- |
| HoprDistributor | a live smart contract on **mainnet** that users can claim HOPR tokens      |
| CLAIMANT_ADDR   | an **ethereum address** that can claim HOPR tokens through HoprDistributor |
| HOPR_ID         | a **HOPR id** of a HOPR node                                               |

## Process

1. User sends an email to tech@hoprnet.org with subject `Link my HOPR node`, and a message containing their `HOPR_ID` and the signature of `Link my Ethereum address to HOPR node address <HOPR_ID>` that was signed using `CLAIMANT_ADDR`. Example (using [MEW](https://www.myetherwallet.com/wallet/sign)):

```
{
  "address": "0xa18732dc751be0db04157eb92c92ba9d0fc09fc5",
  "msg": "0x4c696e6b206d7920457468657265756d206164647265737320746f20484f5052206e6f646520616464726573732031365569753248416d51425a4134547a6a4b6a553566704353707247754d3279386d7065704e774d53365a4b464154694b67363868",
  "sig": "6cc0b880a6dfd9b72b9f8f684d6141c28e3f2395de58e1d3fd304561e798833d1a08e8317a5e6a5bd7585751abda7b30484cbec09492391fb2ccbd575ed7df431c",
  "version": "3",
  "signer": "MEW"
}
16Uiu2HAmQBZA4TzjKjU5fpCSprGuM2y8mpepNwMS6ZKFATiKg68h
```

2. One of the [Representantives](./.processes/development.md#representantives) creates a github issue containing the email's content. Any personally identifyable data should be removed from the email. Issue is treated as a [Fire alarm](./.processes/development.md#fire-alarm).
3. Team member must then verify the validity of the request:

   1. User has indeed signed the right message:

   ```
   ethers = require("ethers")

   ethers.utils.hexlify(ethers.utils.toUtf8Bytes("Link my Ethereum address to HOPR node address 16Uiu2HAmQBZA4TzjKjU5fpCSprGuM2y8mpepNwMS6ZKFATiKg68h"))
   // 0x4c696e6b206d7920457468657265756d206164647265737320746f20484f5052206e6f646520616464726573732031365569753248416d51425a4134547a6a4b6a553566704353707247754d3279386d7065704e774d53365a4b464154694b67363868
   ```

   2. Verify signature [MEW](https://www.myetherwallet.com/wallet/verify)
   3. Signer address (aka `CLAIMANT_ADDR`) of the signature has claimable tokens on schedules `EarlyTokenBuyers` or `TeamAndAdvisors` _(link to etherscan method)_.
   4. Signer does not already exist in [unreleasedTokens.json](./packages/cover-traffic-daemon/src/unreleasedTokens.json)
   5. Inserted `HOPR_ID` is a valid HOPR ID _(link to tooling)_

4. Team member adds another entry to under `link` in [unreleasedTokens.json](./packages/cover-traffic-daemon/src/unreleasedTokens.json) as

```
  hoprId: [
    ethAddress1,
    ethAddress2
  ]
```

5. Team member creates a PR so another team member can review and approve, from here on we follow our usual [DEVELOPMENT_PROCESSES](./.processes/development.md) steps, reviewer must verify the validity of the request as well.
6. Once merged, team member replies to user with:

```
Hello,

Your HOPR ID has been added into the list.
```

## Notes for tech

- A live **mainnet** instance has not been deployed yet, this means that the discribed process cannot be followed through yet.
- Test [unreleasedTokens.spec.ts](./packages/cover-traffic/unreleasedTokens.json) validates entries on each PR, however it does not validate if the `CLAIMANT_ADDR` has claimable tokens. This is topic we will have to tackle once tokens become claimable.
- While we may run HOPR in xDai, Goerli, or other networks, `unreleasedTokens` source of truth lives in a **mainnet** HoprDistributor contract. These tokens should only affect importance calculation when HOPR is running on `xDai` or `mainnet`, else it can be ignored.
- **mainnet** HoprDistributor contract can be updated. Therefore, when the tech team is made aware of such updates, `allocation` field inside the `unreleasedTokens.json` should be updated at the same time. To update this field, please download the result from the [Dune query](https://dune.xyz/queries/237335/444204) and use the [parsing tool](https://github.com/hoprnet/hopr-devrel/tree/parse-unrelased-token/parse-unreleased-token) to get the correct format.

## Notes for community

- User should expect the entry to be added within 1-2 working days, if it's not added, user should reach us through telegram.
