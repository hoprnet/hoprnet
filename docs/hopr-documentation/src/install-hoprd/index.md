# Start Here

```eval_rst
.. ATTENTION::
   The HOPR client software (hoprd) is still under heavy development. Please do *not* add funds to the node that you cannot lose.

   For further questions, please visit our `Telegram channel <https://t.me/hoprnet>`_.
```

To use the HOPR network, you will need a HOPR node. Currently, the only way to do so is running `hoprd`, a node.js process that implements the HOPR protocol and effectively transforms the device you are running it in into a HOPR node. Please bear in mind that by simply installing `hoprd`, you are not making your computer a HOPR node. It is required you run the service as an application and have a working internet connection.

There are several ways to run `hoprd`: you can run it in your own device, install it on a virtual private server (VPS) or use a dedicated hardware device such as the AVADO HOPR Node PC, which has it as a package (Docker image).

## For VPS users

If you’re running your nodes on a VPS, make sure you’ve logged in to your server with the port forwarding feature.

```
ssh -L 3000:127.0.0.1:3000 <root or username>@<Your_server_ip>
```

`<root or username>` - replace with your server username

`<Your_server_ip>` - replace with your server IP

Example: `ssh -L 3000:127.0.0.1:3000 root@192.168.0.1`

## hoprd installation methods

We support multiple distribution mechanisms to install a `hoprd`:

- **(recommended)** **[hopr-sh](using-script.md)**: An automated script able to install all the dependencies on your operating system alongside `hoprd`.
- **[avado](using-avado.md)**: An [AVADO](https://ava.do/) plug-n-play device able to install a HOPR node as a DappNode package from their store.
- **[npm](using-npm.md)**: The popular [Node Package Manager](https://www.npmjs.com/) (npm), which requires node.js.
- **[docker](using-docker.md)**: Using [Docker](https://www.docker.com/) you can run a `hoprd` within a container with a shared volume to store your node info.

Regardless of which way you install `hoprd`, you will access it and interact with it through your browser. By default, `hoprd` exposes an admin interface available on `localhost:3000`, although flags can change these settings.
