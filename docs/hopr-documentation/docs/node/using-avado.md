---
id: using-avado
title: Using An Avado Node
---

To install your AVADO Node, follow the instructions which came in the box. If you have a HOPR PC Node, it will come with HOPR pre-installed.

You will be able to run a HOPR node on your AVADO box.

:::caution Warning
Please delete the old package first, before deleting the package make sure you withdraw all your funds.
:::

While connected to your AVADO network or via a VPN, go to the following [link](http://my.ava.do/#/installer/%2Fipfs%2FQmPhSZTZbM6kd9VizvZpKDN3fQe5bqvCDooCBPYEUXdTcy). Just click the install button and wait until the download completes, this can take some time. 

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

HOPR node running on Avado can be configured to use custom RPC provider. If you're running an ETH node client on your Avado,
you can use it's RPC endpoint URL and paste it into `HOPRD_PROVIDER` environment variable on the configuration page.

Updating the value will restart your HOPR node on Avado, and point it to your ETH client.

**WARNING:** HOPR is currently using Gnosis chain (formerly xdai). If your ETH client is setup with a different chain, HOPR node will not work!
