---
id: how-to-stake
title: Staking on HOPR smart contract
---

The goal of the HOPR staking scheme is to both reward HOPR token holders and incentivize node runners. This staking scheme takes standard yield farming and gives a gamified twist thanks to collectable NFTs, which can boost your APR. 

Locking HOPR tokens in this smart contract isn’t just a way to farm tokens; it’s your gateway to participating in node running and early product access.

## Base Rewards

To participate in HOPR staking, you’ll need to stake and lock your tokens in a HOPR smart contract.

Rewards will pay out as the contract runs. You can claim your rewards as you go, or you can choose to reinvest them in the contract to compound your earnings. If you choose to reinvest, your APR will increase, but those tokens will be locked until the end of the current staking season, just like the rest of your stake.

Staked tokens receive a base reward of 0.00342465753% per day. This works out at 1.25% APR, assuming you don’t take advantage of compounding.

To maximize your staking rewards, you'll need to accumulate APR-boosting NFTs.

## NFT Boosts

HOPR NFTs are a way to track participation in node running and community events, such as one-off games and promotions.

Linking that NFT to your staking address will earn you an APR boost. Best of all, they stack, so the more NFTs you collect the higher your APR will be.

Want to know more about the staking? [Read here](https://medium.com/hoprnet/hopr-staking-faqs-780edfd4f1e1)

## How to stake?

:::info

The HOPR staking contract lives on Gnosis chain, and stakes are in xHOPR. You need to make sure your wallet is set to Gnosis chain before you connect to the widget. If you try to connect on Ethereum, nothing will happen.

:::

If you’re using Metamask, you can find a tutorial for [adding xDAI here](https://www.xdaichain.com/for-users/wallets/metamask/metamask-setup). If you need to know how to convert HOPR on ETH to xHOPR, or unwrap wxHOPR to xHOPR, you can find that [here](convert-hopr)

:::tip

Where to get HOPR tokens? You will find details [here](how-to-get-hopr).

Want to convert HOPR with xHOPR or xHOPR to HOPR? You will find details [here](convert-hopr)

Want to unwrap from wxHOPR to xHOPR or wrap from xHOPR to wxHOPR? You will find details [here](convert-hopr)

:::

Once you have the xHOPR you want to stake in your wallet, and your wallet connected to Gnosis chain, you can go ahead and connect to the HOPR staking interface.

Visit [https://stake.hoprnet.org](https://stake.hoprnet.org) You’ll see the staking interface.

![Connect Wallet](/img/staking/Staking_New_1.png)

(**1**) Press the “Connect to a wallet” button in the top right.

![Choose Wallet](/img/staking/Staking_New_2.png)

Choose your wallet type. For this tutorial we’ll use MetaMask.

![Staking on HOPR smart contract](/img/staking/staking-3.png)

A popup will appear.

(**2**) Select the wallet you want to use as your staking address from the list

(**3**) After you selected your staking address, click “Next”.

![Staking on HOPR smart contract](/img/staking/staking-4.png)

(**4**) Another popup will appear. Click “Connect” to give the staking interface permission to access your balance details.

![Account details](/img/staking/Staking_New_3.png)

(**5**) Once connected, the site will update to show your personal staking details. The wxHOPR, xHOPR and xDAI balances of your wallet will show. In our example there’s 10 xHOPR available to stake.

(**6**) Enter the amount you want to stake in the amount field.

(**7**) After you entered the amount, press the “Stake” button.

![Confirm Stake](/img/staking/Staking_New_4.png)

(**8**) Another popup will appear. Press “Confirm” to stake your tokens. You’ll need some xDAI to pay the extremely small fee associated with this. (In this example, the fee was less than a cent.)

Wait for the transaction to confirm. This should take just a few seconds, but it may take several minutes if the chain is busy. Once the transaction is confirmed, your balances should update.

![Results](/img/staking/Staking_New_5.png)

(**9**) As you can see in the example bellow, the available xHOPR balance is now 0, and the staked balance is now 10.

## Earning and Claiming Rewards

Rewards can be claimed at any time. To claim, click the “Claim Rewards” button. Like staking, there will be a small fee associated with this.

Rewards are paid in wxHOPR, you can unwrap your wxHOPR to xHOPR using our wrapper. Once unwrapped, the xHOPR can be immediately staked by repeating the steps above, you can find that [here](convert-hopr)

![Staking on HOPR smart contract](/img/staking/staking-8.png)

First, you can go to the UI at [https://stake.hoprnet.org](https://stake.hoprnet.org) and connect your wallet, as explained above. If your wallet has any NFTs in it, they will appear in the NFT panel.

Second, you can visit blockscout.com and enter your address. Click the “Token” tab and your NFTs will appear.

## Redeeming NFTs

If you want to redeem an NFT, head to [https://stake.hoprnet.org](https://stake.hoprnet.org) and connect your wallet.
To redeem an NFT, press the “Lock NFT” button under the NFT you want to lock. A popup will appear asking you to confirm and pay a (very) small amount of gas.

:::caution Important

When you confirm, the NFT will be sent to the token contract and boost the associated address. This cannot be undone! So make sure you’re redeeming the NFT you want in the address you want. Like stakes, NFTs are locked for a three-month period, after which they can be claimed.

:::
