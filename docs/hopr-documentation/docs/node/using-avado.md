---
id: using-avado
title: Using An Avado Node
---

To install your AVADO Node, follow the instructions which came in the box. If you have a HOPR PC Node, it will come with HOPR pre-installed.

You will be able to run a HOPR node on your AVADO box.

:::caution Warning
Please delete the old package first, before deleting the package make sure you withdraw all your funds.
:::

While connected to your AVADO network or via a VPN, go to the following [link](http://my.ava.do/#/installer/%2Fipfs%2FQmPhSZTZbM6kd9VizvZpKDN3fQe5bqvCDooCBPYEUXdTcy). This will show a new package version. Just click the install button and wait until you see the success message. This may take some time.

If you are unable to use the link above, search for this hash in the AVADO DappStore:

```
/ipfs/QmPhSZTZbM6kd9VizvZpKDN3fQe5bqvCDooCBPYEUXdTcy
```

Alternatively, you can click the following [link](http://my.ava.do/#/installer/%2Fipfs%2FQmPhSZTZbM6kd9VizvZpKDN3fQe5bqvCDooCBPYEUXdTcy).

![DappStore](/img/node/avado-1.png)

After you installed the HOPR package, please go to `My DApps` section and click on the HOPR client.

![MyDapps](/img/node/avado-2.png)

If the HOPR client will ask you to enter the **security token**, enter this fixed phrase: `!5qxc9Lp1BE7IFQ-nrtttU`

The installation process has been finished! Now you can proceed to [Guide using a hoprd node](guide-using-a-hoprd-node).

## Additional HOPR configuration on Avado

HOPR node running on Avado can be configured to use custom RPC provider. If you're running an ETH node client on your Avado,
you can use it's RPC endpoint URL and paste it into `HOPRD_PROVIDER` environment variable on the configuration page.

Updating the value will restart your HOPR node on Avado, and point it to your ETH client.

**WARNING:** HOPR is currently using Gnosis chain (formerly xdai). If your ETH client is setup with a different chain, HOPR node will not work!
