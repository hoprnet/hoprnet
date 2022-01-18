---
id: convert-hopr
title: Converting HOPR token
---

import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';

Like DAI and xDAI, HOPR and xHOPR exist in a 1:1 relationship on xDAI Chain and Ethereum mainnet, respectively. To convert between them, you need to use a tool called a Omnibridge found at [https://omni.xdaichain.com](https://omni.xdaichain.com)

<Tabs>
<TabItem value="xhtoh" label="xHOPR to HOPR">

:::tip

Your wallet needs to be connected to xDAI Chain. If you haven’t set that up yet, you can find the details [here](https://www.xdaichain.com/for-users/wallets/metamask/metamask-setup). You’ll need to select “Add Custom RPC” in your wallet and fill in the information.

:::

Click the “Connect” button to connect your wallet.

![Convert xHOPR to HOPR](/img/convertion/xh_h-1.png)

Choose your wallet type. For this tutorial we’ll use MetaMask.

![Convert xHOPR to HOPR](/img/convertion/xh_h-2.png)

A popup will appear. Select the wallet you want to use and click "Next".

![Convert xHOPR to HOPR](/img/convertion/xh_h-3.png)

1. You’ll see a screen with your wallet address and xDAI at the top right.
2. Now you need to select your tokens to swap. Select the dropdown menu on the “From” side of the bridge.

![Convert xHOPR to HOPR](/img/convertion/xh_h-4.png)

3. Search for HOPR token and click on it. After selecting HOPR token, it will say “HOPR Token on xDAI” on the “From” side and “HOPR Token” will be automatically selected on the “To” side.

![Convert xHOPR to HOPR](/img/convertion/xh_h-5.png)

Enter the amount you want to transfer across the Omnibridge. The amount you will receive will be shown on the other side. This will be very slightly different, because the bridge charges a small fee.

Press “Request” to begin the transfer.

![Convert xHOPR to HOPR](/img/convertion/xh_h-6.png)

You will receive a warning confirming the bridge fee for the transfer and explaining about the gas fees involved. There will be two transactions, one on the xDAI side and one on the ETH side. Each will cost gas, so make sure you have both currencies in your wallet, in addition to your xHOPR.

Once you’re ready to start the transfer, press “Continue”

![Convert xHOPR to HOPR](/img/convertion/xh_h-7.png)

You need to wait for 12 block confirmations before the transfer begins. This will happen on the xDAI side so should only take around a minute.

If you click the blue link, you’ll see what’s going on behind the scenes.

:::caution Warning

Don’t close the Omnibridge tab! Your transaction won’t be lost, but it will require a bit more effort to process manually. If this happens, please contact the ambassadors in our Telegram channel for help.

:::

![Convert xHOPR to HOPR](/img/convertion/xh_h-8.png)

Once the transaction reaches 12 block confirmations, an automated multisig will process the transaction on the ETH side. Four out of seven validators must provide signatures to approve the transfer. This can take a little time.

![Convert xHOPR to HOPR](/img/convertion/xh_h-9.png)

Your tokens will need to be claimed on the other side of the bridge. To do this, your wallet will need to be disconnected from xDAI Chain and connected to the Ethereum mainnet.

In the Omnibridge, you’ll see a popup asking you to do just that:

![Convert xHOPR to HOPR](/img/convertion/xh_h-10.png)

Change back to ETH Mainnet in your wallet, and the Omnibridge will automatically detect it.

The popup will change to show a “CLAIM” button:

![Convert xHOPR to HOPR](/img/convertion/xh_h-11.png)

Press “Claim” and a popup will appear from your wallet asking you to confirm the transaction on the ETH Mainnet. This will be a lot more expensive than the transaction on the xDAI side, and may take some time due to congestion. But once it confirms, you will have HOPR in your wallet on ETH Mainnet!

</TabItem>
<TabItem value="htoxh" label="HOPR to xHOPR">

Converting from HOPR to xHOPR works in exactly the same way like from xHOPR to HOPR, using the Omnibridge. There are a few small differences.

- There’s no bridge fee.
- You’ll obviously need to switch which address goes in which box when adding the assets
- There won’t be a “Claim” process like in the xDAI to ETH direction — you’ll just receive your tokens.

If your metamask wallet will be on Ethereum network, then after connecting your wallet with Omnibridge, it will show you on the "From" side "ETH Mainnet" and on the "To" side "xDai chain".

1. Now you need to select your tokens to swap. Select the dropdown menu on the “From” side of the bridge and find a HOPR token.
2. Enter the amount you want to transfer across the Omnibridge. The amount you will receive will be shown on the other side.
3. As with most ERC20 dapps, there will be an extra transaction before you can begin: before you can use the Omnibridge on the ETH side, you’ll need to sign an `approve` transaction with your wallet. Press the “Unlock” button.

![Convert xHOPR to HOPR](/img/convertion/h_xh-1.png)

And confirm the prompt that appears from your wallet.

![Convert xHOPR to HOPR](/img/convertion/h_xh-2.png)

After unlocking, press “Transfer” button to begin the transfer, confirm the transaction. Wait a little bit and your HOPR tokens from ETH network will be transferred to xDai network.

</TabItem>
<TabItem value="xhtowxh" label="xHOPR to wxHOPR">

To convert between xHOPR and wxHOPR you should use the HOPR wrapper, found at [https://wrapper.hoprnet.org](https://wrapper.hoprnet.org)

Switch your wallet to xDAI Chain and press “Connect Wallet”.

![Convert xHOPR to HOPR](/img/convertion/wrapper-1.png)

1. Select the wallet address you want to use from the list
2. Click "Next" and then click "Connect" button.

![Convert xHOPR to HOPR](/img/convertion/wrapper-2.png)

Enter the amount of xHOPR you want to convert (wrap) to wxHOPR and click button "Swap to wxHOPR", confirm the transaction.

![Convert xHOPR to HOPR](/img/convertion/wrapper-3.png)

To convert (unwrap) from wxHOPR to xHOPR repeat same process, but on wxHOPR field, enter amount you would like to convert to xHOPR and press button "Swap to xHOPR".

</TabItem>
</Tabs>
