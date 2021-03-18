<!-- ---
description: 'Learn about why HOPR needs two kinds of tokens to run, and how to get them.'
--- -->

# Native Tokens

In order to participate in the testnet, you'll need to fund your node with at least 0.02 MATIC.

{% hint style="warning" %}
You'll spend a \(very\) small amount of MATIC when you perform actions which interact with the HOPR smart contracts, such as opening and closing payment channels and redeeming tickets after relaying data.

Currently, HOPR Chat doesn't always notify you if an action has failed due to lack of funds. So if things aren't behaving as expected use `balance` to check you aren't low on MATIC. More notifications will be added in future versions.
{% endhint %}

## Getting MATIC

The easiest way to get MATIC is to ask one of our ambassadors in [**Telegram**](https://t.me/hoprnet) or [**Discord**](https://discord.gg/dEAWC4G).

To get MATIC for yourself, you'll need to add the network to your Ethereum wallet. [MetaMask](https://metamask.io/) is one of the most widely supported wallets, but there are other options.

To configure the MATIC Network, click the Network button at the top of your Metamask wallet.

![](../../images/matic-metamask-1.png)

A drop-down menu will appear. Click "Custom RPC"

![](../../images/matic-metamask-2.png)

A form will appear. Fill in the fields with the following information:

- **Network Name:** Matic Mainnet
- **New RPC URL:** [https://rpc-mainnet.matic.network](https://rpc-mainnet.matic.network)
- **ChainID:** 137
- **Symbol:** MATIC
- **Block Explorer URL:** [https://explorer.matic.network](https://explorer.matic.network)

To get MATIC, you can use the MATIC bridge at [https://wallet.matic.network/](https://wallet.matic.network/) to convert them from gETH. You can also use a tool such as [Uniswap](https://app.uniswap.org).

Once you have MATIC in your Metamask wallet, send some to your node wallet, at the address shown when your node starts up.

Once you've sent MATIC to your node, restart it. When your node restarts, your balance will be automatically detected and you can proceed.
