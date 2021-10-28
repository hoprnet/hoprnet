# Using NPM

The NPM setup allows you to install and run **HOPRd** as a Node.js application, ensuring your experience is a close as to the developer’s have when developing **HOPRd** and the **HOPR Core** protocol. Node.js might require further software installation, but is able to be run in less hardware demanding machines, while taking considerable less space in comparison to Docker \(i.e. 50mb\).

## Installing NPM

In order to get NPM on your machine, you will need to install Node.js, our recommended way of doing this is to install [nvm](https://github.com/nvm-sh/nvm) / [nvm-windows](https://github.com/coreybutler/nvm-windows), a Node.js version manager. This ensures we can install and uninstall as many versions of Node.js as needed. Furthermore, it will help you installing any additional requirements \(if any\) for running Node.js.

```eval_rst
.. content-tabs::

    .. tab-container:: windows
        :title: Windows

        `Windows Guide <https://github.com/coreybutler/nvm-windows#install-nvm-windows>`_

    .. tab-container:: linux_macos
        :title: Linux or macOS

        `Linux & macOS Guide <https://github.com/nvm-sh/nvm#installing-and-updating>`_
```

_Please bear in mind you might need to restart your terminal after running these commands._

## Installing Node.js

After you have downloaded and setup nvm in your machine \(run `nvm ls` to ensure everything is in place\), now you need to install a specific version of Node.js before running **HOPRd**.

At the time of writing, **HOPRd** runs on Node.js `v16`. Specifically, **HOPRd** has been developed and tested in `v16`, so in case you run on any issues with **HOPRd,** try switch to `v16` to see if those issues disappear.

To install Node.js with nvm, run the following

```bash
$ nvm install 16
$ nvm use 16
```

If everything was done properly, you can run `node --version` to see your current `node` version, alongside running basic commands as shown when running simply `node` in your terminal.

```eval_rst
.. ATTENTION::
   MacOS M1 users will need to follow an extra set of instructions from `NVM <https://github.com/nvm-sh/nvm#macos-troubleshooting>`_ to allow them to use Node.js 16.
```

![node](https://user-images.githubusercontent.com/73285987/139115268-01aef9bb-d473-40d1-b291-864a3c2b7471.gif)


## Installing HOPRd using NPM

```bash
$ mkdir hopr-wildhorn-v2
$ cd hopr-wildhorn-v2
$ npm install @hoprnet/hoprd@wildhorn-v2

# run hoprd
$ DEBUG="hopr*" npx hoprd --init --admin --rest --identity ./hoprd-id-01 --data ./hoprd-db-01 --password='hopr-01' --testNoAuthentication

# add security
$ DEBUG="hopr*" npx hoprd --init --admin --rest --identity ./hoprd-id-01 --data ./hoprd-db-01 --password='hopr-01' --apiToken='<YOU_SECRET_TOKEN>'

Please note that if `--rest` or `--admin` is specificed, you **must** provide an `--apiToken` which is at least 8 symbols, contains a lowercase and an uppercase letter, a number and a special symbol. This ensures the node cannot be accessed by a malicious user residing in the same network.
```
