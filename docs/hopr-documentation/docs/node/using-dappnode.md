---
id: using-dappnode
title: Using Dappnode
---

To set up your DAppNode, follow the instructions that came with the box. Then, just install the HOPR client and you can start using your node right away!

:::caution Warning
Please withdraw all your funds and delete the old package before installing a new one.
:::

## Installing the HOPR Client

(**1**) Open the DAppStore using the sidebar to the left. Then at the top of the DAppStore, toggle on the public repository bar.

(**2**) A warning prompt will appear. Press `I understand, take me to the public repo` to continue.

**Note:** You may see an empty directory after going to the public repository, just refresh the page if this happens. You may need to refresh several times until you see several packages in your directory, one of them being `Hopr`.

![DappStore](./images/DappStore-NR.png)

(**3**) You should see the latest version of the HOPR client now. Just click `Get` and wait for it to install

That's all! You should now be able to find the HOPR client in your 'Packages'.

![MyDapps](./images/Dappnode-2.png)

Simply open the client, and you should be greeted with the hopr-admin interface.

Your **security token** is set to `!5qxc9Lp1BE7IFQ-nrtttU`. You will need this to access hopr-admin.

If you are in the process of registering your node on the network registry, please complete the process [here](./network-registry-tutorial.md) before continuing.

Otherwise, the installation process is complete! You can proceed to our [hopr-admin tutorial](using-hopr-admin).

### Restoring an old node

If you have previously installed a node and have the [identity file downloaded](using-hopr-admin#backing-up-your-identity-file), you can use it to restore your old node.

**Note:** For DAppNode, you should download the latest version of HOPR before trying to restore your node.

Find HOPR in your packages and navigate to the backup section. From there, all you have to do is click 'Restore' and open your [zipped backup file](using-hopr-admin#backing-up-your-identity-file) when prompted.

![dappnode restore](./images/dappnode-backup.png)

## Collecting Logs

If your node crashes, you will want to collect the logs and pass them on to our ambassadors on telegram or create an issue on GitHub.

To collect the logs:

(**1**) Find HOPR in your packages and navigate to the backup section.

![Dappnode Logs](./images/Dappnode-logs.png)

(**2**) From there, all you have to do is click 'Download all'.

Using the downlaoded file either:

- Send it to an ambassador on our [telegram](https://t.me/hoprnet) for assistance.
- Or, create an issue using our bug template on [GitHub.](https://github.com/hoprnet/hoprnet/issues)

## Using a Custom RPC Endpoint

You can set your own RPC endpoint for HOPR to use. Ideally, you would install an ETH client on your DAppNode and use its local provider. A local provider helps increase decentralisation and is generally good practice, but you can also use any RPC provider of your choice.

### Finding your local endpoint

If you have already installed an ETH client, you can find its RPC endpoint on the package's info page.

![ETH client settings](./images/RPC-endpoint-Dappnode.png)

The image above shows the RPC endpoint for the GETH client (querying API in the image): `http://ethchain-geth.my.ava.do:8545`. Your endpoint will be different depending on the client you have installed. Otherwise, you can use any non-local RPC provider such as [ankr.](https://www.ankr.com/)

### Changing your RPC endpoint

To change your RPC endpoint:

(**1**) Find HOPR in your packages and navigate to the 'Config' section.

![RPC Prpvider Dappnode](./images/HOPR-provider-Dappnode.png)

(**2**) Paste your custom RPC endpoint in the text field under `RPC Provider URL`.

(**3**) Click 'Update' and wait for your node to restart.

All done! Your DAppNode node will now use your specified RPC endpoint.
