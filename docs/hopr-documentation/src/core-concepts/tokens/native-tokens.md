<!-- ---
description: 'Learn about why HOPR needs two kinds of tokens to run, and how to get them.'
--- -->

# Native Tokens

In order to participate in the testnet, you'll need to fund your node with at least 0.02 ETH.

```eval_rst
.. WARNING::
   You'll spend a \(very\) small amount of Ropsten ETH when you perform actions which interact with the HOPR smart contracts, such as opening and closing payment channels and redeeming tickets after relaying data.

   Currently, HOPRd doesn't always notify you if an action has failed due to lack of funds. So if things aren't behaving as expected use `balance` to check you aren't low on ETH. More notifications will be added in future versions.
```

## Getting Ropsten ETH

The easiest way to get Ropsten ETH is to try to claim them by using one of these faucets:

- https://faucet.ropsten.be/
- https://faucet.metamask.io/
- https://faucet.dimensions.network/

You will need to make sure that in your metamask wallet you select the Ropsten network.

Once you have Ropsten ETH in your Metamask wallet, send some to your node wallet, at the address shown when your node starts up.

Once you've sent Ropsten ETH to your node, restart it. When your node restarts, your balance will be automatically detected and you can proceed.
