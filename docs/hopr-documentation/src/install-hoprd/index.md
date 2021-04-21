# Start Here

```eval_rst
.. ATTENTION::
   The HOPR client software is still under development. Please do *not* add funds to the node that you cannot lose.

   For further questions, please visit our `Telegram channel <https://t.me/hoprnet>`_.
```

To use the HOPR network, you will need a HOPR node. There are several ways to run a HOPR node: you can use your own device, install one on a virtual private server (VPS) or use a dedicated hardware device such as the AVADO HOPR Node PC.

We support multiple distribution mechanisms to install a HOPR node:

- **[hopr-sh](using-script.md)**: An automated script able to install all the dependencies on your operating system alongside a HOPR node.
- **[avado](using-avado.md)**: An [AVADO](https://ava.do/) plug-n-play device able to install a HOPR node as a DappNode package from their store.
- **[npm](using-npm.md)**: The popular [Node Package Manager](https://www.npmjs.com/) (npm), which requires node.js.
- **[docker](using-docker.md)**: Using [Docker](https://www.docker.com/) you can run a HOPR node within a container with a shared volume to store your node info.


Regardless of which way you install your HOPR node, you will access it and interact with it through your browser. By default, a hopr node exposes an admin interface available on `localhost:3000`, although flags can change these settings.