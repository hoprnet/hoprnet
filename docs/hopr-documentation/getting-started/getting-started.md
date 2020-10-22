---
description: Getting up to speed from previous versions
---

# Previous Users

If you've already run HOPR Chat before, you can update to the latest version in no time. Please bear in mind that features and commands may have changed since you last used the network. If in doubt, type `help` to see a list of available commands.

{% hint style="info" %}
If this is your first time using HOPR Chat, you'll need to install node.js or Docker. we recommend following the [HOPR Chat Quickstart Guide](https://docs.hoprnet.org/home/getting-started/hopr-chat/quickstart) to do this.
{% endhint %}

## Step 1: Connect to the HOPR Network

There are two ways to connect to the testnet:

- With the HOPR Chat app
- Using your AVADO node \(either a plug-and-play HOPR Node PC or a separate model purchased direct from AVADO\)

### Connecting Using HOPR Chat

To connect using HOPR Chat, you'll need to download the latest version.

{% tabs %}
{% tab title="Windows" %}
[Download the latest Windows release](https://github.com/hoprnet/hopr-chat/releases/download/v1.17.0-alpha.basodino.rc-1/hopr-chat-nodebin-windows.zip)
{% endtab %}

{% tab title="MacOS" %}
[Download the latest MacOS release](https://github.com/hoprnet/hopr-chat/releases/tag/v1.17.0-alpha.basodino.rc-1)
{% endtab %}

{% tab title="Linux" %}
[Download the latest Linux release](https://github.com/hoprnet/hopr-chat/releases/download/v1.17.0-alpha.basodino.rc-1/hopr-chat-nodebin-linux.zip)
{% endtab %}
{% endtabs %}

If you want to keep your address from a previous version, copy the `db` folder before installing the latest version. After installing the latest version, paste it back into the `hopr-chat`folder and your address \(and any funds\) will be restored.

**Connecting Using an AVADO Node**

If you have an AVADO Node, it should automatically update to the latest public version of HOPR. After updaing, all you have to do is fund your node, as explained in the next section, and then restart.

![](../.gitbook/assets/avado-no-funds%20%282%29%20%281%29.png)

{% hint style="info" %}
If you've been invited to test a development build using your AVADO node, select the DappStore in the left-hand menu of your AVADO dashboard and paste the IPFS hash into the search bar. You will see an option to update your HOPR dApp. Click it, and wait for it to install.
{% endhint %}

## Step 2: Fund Your Node

Next, you need to send xDAI to your HOPR node. You currently need 0.02 xDAI in your node address to participate in the testnet. If your node doesn't have enough xDAI, HOPR Chat will not start.

{% hint style="info" %}
xDAI is a USD stablecoin, so 0.02 xDAI is worth around 2 cents. It costs xDAI to open payment channels and perform certain other testnet actions, but 0.02 is more than enough.
{% endhint %}

If your node is unfunded, you can find your xDAI address by simply starting the HOPR Chat client. HOPR Chat will recognize that your node is unfunded, and won't proceed. It will tell you your address, so you can send xDAI. Once your node is funded, you can find your address by typing `myAddress`.

![](../.gitbook/assets/no-funds%20%283%29%20%281%29.png)

{% tabs %}
{% tab title="First Tab" %}

{% endtab %}

{% tab title="Second Tab" %}

{% endtab %}
{% endtabs %}

If you need more instructions on how to buy and send xDAI, see the **Funding Your Node** section

Otherwise, please proceed to the [**CoverBot**](coverbot.md) section.
