---
id: using-avado
title: Using an Avado Node
---

To set up your AVADO Node, follow the instructions in the box. If you have a HOPR PC Node, it will come with HOPR pre-installed. Simply, download the HOPR client to start interacting with your node. Simply download the HOPR client to start interacting with your node!

:::caution Warning
Please delete any old packages as necessary; make sure to withdraw your funds before doing so.
:::

While connected to your AVADO network or via a VPN, go to the following [link](http://my.ava.do/#/installer/%2Fipfs%2FQmPhSZTZbM6kd9VizvZpKDN3fQe5bqvCDooCBPYEUXdTcy). Just click the install button and wait until the download completes.

If you are unable to use the link above, search for this hash in the AVADO DappStore:

```
/ipfs/QmPhSZTZbM6kd9VizvZpKDN3fQe5bqvCDooCBPYEUXdTcy
```

Alternatively, you can click the following [link](http://my.ava.do/#/installer/%2Fipfs%2FQmPhSZTZbM6kd9VizvZpKDN3fQe5bqvCDooCBPYEUXdTcy).

![DappStore](/img/node/avado-1.png)

After you have installed the HOPR package, you can find the HOPR client in `my DApps`.

![MyDapps](/img/node/avado-2.png)

If the HOPR client asks you to enter a **security token**, paste the following into the command line: `!5qxc9Lp1BE7IFQ-nrtttU` and hit enter.

The installation process is now complete! You can proceed to our [hopr-admin tutorial](using-hopr-admin).  

## Additional HOPR configuration on Avado

The HOPR client on Avado can be configured to use a custom RPC provider. If you're running an ETH node on your Avado,
you can copy its RPC endpoint and paste it into the `HOPRD_PROVIDER` environment variable on the HOPR client configuration page.

Updating this value will restart your node and point it to your ETH client.

**WARNING:** HOPR is currently using the Gnosis chain (formerly xDAI). If your ETH client is set up with a different chain, the HOPR node will not work!
