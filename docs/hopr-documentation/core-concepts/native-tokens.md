---
description: 'Learn about why HOPR needs two kinds of tokens to run, and how to get them.'
---

# Native Tokens

To use HOPR, you'll need two types of token: 

* a HOPR token
* the native token of the blockchain the HOPR token is linked to

After mainnet launch, the native token will be ETH and the HOPR token will simply be HOPR. However, for our various testnets HOPR will be running on a variety of different chains.  
  
The HOPR testnet is currently running on xDAI Chain, a sidechain of Ethereum, so the native token is xDAI. The HOPR token for the testnet is xHOPR.  
  
In order to participate in the testnet, you'll need to fund your node with at least 0.02 xDAI \(xDAI is a stablecoin, so this is worth around $0.02\). Your node won't start unless it's sufficiently funded.

{% hint style="warning" %}
You'll spend a \(very\) small amount of xDAI when you perform actions which interact with the HOPR smart contracts, such as opening and closing payment channels and redeeming tickets after relaying data.  
  
Currently, HOPR Chat doesn't always notify you if an action has failed due to lack of funds. So if things aren't behaving as expected use `balance` to check you aren't low on xDAI. More notifications will be added in future versions.
{% endhint %}

### Getting xDAI

To get xDAI, first you'll need to load some ETH into your Ethereum wallet. [MetaMask](https://metamask.io/) is one of the most widely supported wallets, but there are other options.

Next, you'll need to convert some ETH into xDAI. There are several tools you can use to do this. The simplest is to connect your wallet to the tool at [xdai.io](https://xdai.io), where you can swap ETH to DAI and then DAI to xDAI.

![](../.gitbook/assets/xdai-burner%20%282%29.png)

Finally, you'll need to send some xDAI to your node. Because xDAI is a separate chain, you'll need to change the network settings in MetaMask. [The xDAI docs have a step-by-step guide for this](https://www.xdaichain.com/for-users/wallets/metamask/metamask-setup).

Once you've sent xDAI to your node, restart **HOPR Chat**. When your node restarts, your balance will be automatically detected and you can proceed.  


### Getting xHOPR

There are two ways to get xHOPR on your node: sending it directly to the node address or earning it by relaying data and redeeming tickets. For testing purposes, we recommend funding your node directly. Ask in Telegram or Discord and one of our ambassadors will fund your node.  
  
To learn more about earning xHOPR by relaying data, see the sections on [**Multi-hop messages**](../hopr-avado-node-tutorial/sending-a-multi-hop-message.md) and [**Tickets**](../hopr-chat-tutorial/redeeming-tickets.md).

