---
id: start-here
title: Start here
---
# Start here

:::caution Warning
The HOPR client software (hoprd) is still under heavy development. Please do _not_ add funds to the node that you cannot lose.

For further questions, please visit our [Telegram channel](https://t.me/hoprnet)
:::

To use the HOPR network, you will need a HOPR node. Currently, the only way to do so is running `hoprd`, a node.js process that implements the HOPR protocol and effectively transforms the device you are running it in into a HOPR node. Please bear in mind that by simply installing `hoprd`, you are not making your computer a HOPR node. It is required you run the service as an application and have a working internet connection.

There are several ways to run `hoprd`: you can run it on your own device, install it on a virtual private server (VPS) or use a dedicated hardware device such as the AVADO/Dappnode HOPR Node PC, which have it as a package (Docker image).

## Network Registry

If you are using the Monte Rosa release you will not be able to interact with other nodes unless you have been added to the network reistry. You can view the current porcess and details for this here. Please don't join the network expecting to be able to interact wth other ndodes without having first been added to the registry.

## Hardware requirements

The minimum requirements for running `hoprd` on your device:

* Dual Core CPU ~ 2 GHz
* 4 GB RAM
* at least 3 GB Disk Space

Although it is recommended you have at least 8 GB of RAM and 10 GB of disk space.

## For VPS users

It is recomended to use a linux or macOS based VPS if you are on Windows, as Windows is not completely supported. If you plan to run your node on a VPS, make sure you setup port forwarding as shown below:

```
ssh -L 3000:127.0.0.1:3000 <root or username>@<Your_server_ip>
```

`<root or username>` - replace with your server username

`<Your_server_ip>` - replace with your server IP

Example: `ssh -L 3000:127.0.0.1:3000 root@192.168.0.1`

This is so you can access the admin interface locally once the node is running.

## For Linux/macOS users

If you install your node through Docker it will by default only run until you close your terminal. It is highly recommended that you use tmux or screen in order to run your node in the background. This will allow you to create multiple terminal windows that exist as their own independantly running instances. All of which exist on a seperate session that will keep running after you have closed your terminal.

If you intend to use your device to install and run a `hoprd` node please familiarise yourself with [tmux](https://linuxize.com/post/getting-started-with-tmux/) or [screen](https://linuxize.com/post/how-to-use-linux-screen/) and create a new session before continuing.

## hoprd installation methods

We support multiple distribution mechanisms to install a `hoprd`:

**[Avado](using-avado)**

An [AVADO](https://ava.do/) plug-n-play device able to install a HOPR node as a DappNode package from their store.

**[Docker](using-docker)**

Using [Docker](https://www.docker.com/) you can run `hoprd` within a container with a shared volume to store your node info.

Use Docker if you intend to install the node on your device. Regardless of which way you install `hoprd`, you will access it and interact with it through your browser. By default, `hoprd` exposes an admin interface available on `localhost:3000`, although flags can change these settings.
