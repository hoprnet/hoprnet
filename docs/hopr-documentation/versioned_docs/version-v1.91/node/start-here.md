---
id: start-here
title: Start here
---

# Start here

:::caution Warning
The HOPR client software (hoprd) is still under heavy development. Please do _not_ add funds to the node you cannot lose.

For further questions, please visit our [Telegram channel](https://t.me/hoprnet)
:::

To use the HOPR network, you will need to install a HOPR node. Currently, the only way to do so is by running `hoprd`, a node.js process that implements the HOPR protocol and effectively transforms the device you are running it on into a HOPR node. Please bear in mind that by simply installing `hoprd`, you are not making your computer a HOPR node. It is required you run the service as an application and have a working internet connection.

There are several ways to run `hoprd`: you can run it on your device, on a virtual private server (VPS) or use a dedicated hardware device such as the AVADO/Dappnode HOPR Node PC, which has it as a pre-installed package (Docker image).

## Network Registry

If you are using the Monte Rosa environment, you will not be able to interact with other nodes unless you have been added to the network registry. You can view the current process and details for this [here](./network-registry-tutorial.md). If you have been given an NFT and are installing your node to locate your peerID, complete the installation process as normal.

## Hardware requirements

The minimum requirements for running `hoprd` on your device:

- Dual Core CPU ~ 2 GHz
- 4 GB RAM
- at least 3 GB Disk Space

Although it is recommended that you have at least 8 GB of RAM and 10 GB of disk space.

## For VPS users

Using a VPS is recommended if you are on Windows, as all the instructions for installing your node are for Linux/macOS users. A VPS, in general, is an ideal setup as you can use Tmux or Screen to run your node constantly in the background without needing your machine to be plugged in or turned on. If you install your node through Docker, it will only run until you close your terminal for both your local machine and a VPS. This is why it is highly recommended you quickly familiarise yourself with [tmux](https://linuxize.com/post/getting-started-with-tmux/) or [screen](https://linuxize.com/post/how-to-use-linux-screen/) before continuing.

If you intend to run your node locally, try and use a setup where your PC or machine can stay plugged in throughout the day. This is especially important if you are participating in the Monte Rosa release. Otherwise, you can use a plug-n-play device such as Avado or Dappnode, which you can plug in and forget about.

## hoprd installation methods

We support multiple distribution mechanisms to install `hoprd`:

**[Avado](using-avado)**

An [AVADO](https://ava.do/) plug-n-play device, just set it up and install the HOPR package from their Dappstore.

**[Dappnode](using-dappnode)**

A [Dappnode](https://dappnode.io/) plug-n-play device, another quick set-up and installation.

**[Docker](using-docker)**

Using [Docker](https://www.docker.com/) you can run `hoprd` on your device.

Regardless of which way you install `hoprd`, you will access and interact with it through your browser. The exact URL for this will vary depending on your installation method and whether or not you used a VPS. All details can be found in their respective installation section.
