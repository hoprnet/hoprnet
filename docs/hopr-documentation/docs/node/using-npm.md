---
id: using-npm
title: Using NPM
---

import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';

# Using NPM

The NPM setup allows you to install and run **HOPRd** as a Node.js application, ensuring your experience is a close as to the developer’s have when developing **HOPRd** and the **HOPR Core** protocol. Node.js might require further software installation, but is able to be run in less hardware demanding machines, while taking considerable less space in comparison to Docker \(i.e. 50mb\).

## Installing NPM

In order to get NPM on your machine, you will need to install Node.js, our recommended way of doing this is to install [nvm](https://github.com/nvm-sh/nvm) / [nvm-windows](https://github.com/coreybutler/nvm-windows), a Node.js version manager. This ensures we can install and uninstall as many versions of Node.js as needed. Furthermore, it will help you installing any additional requirements \(if any\) for running Node.js.

<Tabs>
<TabItem value="win" label="Windows">

Read more here: [Windows Guide](https://github.com/coreybutler/nvm-windows#install-nvm-windows)

</TabItem>
<TabItem value="linux_mac" label="Linux and macOS">

Read more here: [Linux & macOS Guide](https://github.com/nvm-sh/nvm#installing-and-updating)

</TabItem>
</Tabs>

Please bear in mind you might need to restart your terminal after running these commands.

## Installing Node.js

After you have downloaded and setup nvm in your machine \(run `nvm ls` to ensure everything is in place\), now you need to install a specific version of Node.js before running **HOPRd**.

At the time of writing, **HOPRd** runs on Node.js `v16`. Specifically, **HOPRd** has been developed and tested in `v16`, so in case you run on any issues with **HOPRd,** try switch to `v16` to see if those issues disappear.

To install Node.js with nvm, run the following

```bash
$ nvm install 16
$ nvm use 16
```

If everything was done properly, you can run `node --version` to see your current `node` version, alongside running basic commands as shown when running simply `node` in your terminal.

:::info INFO
   MacOS M1 users will need to follow an extra set of instructions from [NVM](https://github.com/nvm-sh/nvm#macos-troubleshooting) to allow them to use Node.js 16.
:::

![node](/img/node/node.gif)


## Installing HOPRd using NPM

```bash
$ mkdir hopr-wildhorn-v2
$ cd hopr-wildhorn-v2
$ npm install @hoprnet/hoprd@wildhorn-v2
```

## Running HOPRd


### run hoprd
```bash
DEBUG="hopr*" npx hoprd --init --admin --identity ./hoprd-id-01 --data ./hoprd-db-01 --password='hopr-01' --apiToken='<YOUR_SECRET_TOKEN>'
```
:::danger Important

If you want to secure your hoprd admin UI, in the command line you must use **--apiToken** tag. 

**<YOUR_SECRET_TOKEN\>** - Replace it with your own password (don't use "<\>").

Password should contain:
- at least 8 symbols
- a lowercase letter
- uppercase letter
- a number
- a special symbol

This ensures the node cannot be accessed by a malicious user residing in the same network.

:::
