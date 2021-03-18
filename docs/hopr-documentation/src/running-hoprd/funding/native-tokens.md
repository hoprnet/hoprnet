<!-- ---
description: 'Learn about why HOPR needs two kinds of tokens to run, and how to get them.'
--- -->

# Native Tokens

In order to participate in the testnet, you'll need to fund your node with at least 0.025 gETH.

```eval_rst
.. WARNING::
   You'll spend a \(very\) small amount of Goerli gETH when you perform actions which interact with the HOPR smart contracts, such as opening and closing payment channels and redeeming tickets after relaying data.

   Currently, HOPRd doesn't always notify you if an action has failed due to lack of funds. So if things aren't behaving as expected use `balance` to check you aren't low on gETH. More notifications will be added in future versions.
```

The easiest way to get Goerli gETH is to try to claim them by using one of these faucets:

- [goerli.mudit.blog](https://faucet.goerli.mudit.blog/)
- [slock.it](https://goerli-faucet.slock.it/)
- [metamask.io](https://faucet.metamask.io/)

You will need to make sure that in your metamask wallet you select the Goerli network.

When you have Goerli gETH in your Metamask wallet, send some to your node wallet, at the address shown when your node starts up.

Once you've sent Goerli gETH to your node, restart it. When your node restarts, your balance will be automatically detected and you can proceed.
