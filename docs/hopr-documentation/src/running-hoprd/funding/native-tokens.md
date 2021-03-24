<!-- ---
description: 'Learn about why HOPR needs two kinds of tokens to run, and how to get them.'
--- -->

# Native Tokens

In order to participate in the HOPR network, you'll need to fund your node with at least 0.025 xDAI to cover the gas fees.

```eval_rst
.. WARNING::
   You'll spend a \(very\) small amount of xDAI when you perform actions which interact with the HOPR smart contracts, such as opening and closing payment channels and redeeming tickets after relaying data.

   Currently, HOPRd doesn't always notify you if an action has failed due to lack of funds. So if things aren't behaving as expected use `balance` to check you aren't low on xDAI. More notifications will be added in future versions.
```

You can get xDAI by using one of these [methods](https://www.xdaichain.com/for-users/get-xdai-tokens).

You will need to make sure that metamask is running with xDAI. See [here](https://www.xdaichain.com/for-users/wallets/metamask) for more instructions on how to switch Metamask to xDAI.

```eval_rst
.. WARNING::
   Metamask will not show your xDAI as long as you haven't switched to xDAI.
```

When you have xDAI in your Metamask wallet, send some to your node wallet, at the address shown when your node starts up.

```eval_rst
.. ATTENTION::
   The HOPR client is still under development and not all issues are fixed. We recommend to not add more than 10 wxHOPR and 1 xDAI to it.
```

Once you've sent xDAI to your node, restart it. When your node restarts, your balance will be automatically detected and you can proceed.
