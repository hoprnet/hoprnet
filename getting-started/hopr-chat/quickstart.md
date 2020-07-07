---
description: Step and step guide to quickly start using HOPR Chat
---

# Quickstart

## Step 1 - Install node.js

Node.js is JavaScript runtime built on Chrome's V8 JavaScript engine. It allows users to run web applications with the same technology Browsers use to run webpages. **HOPR Chat** is developed using JavaScript, so first we need to install Node.js in your machine.

The easiest way to install and run node.js in your machine, is using a “Node Version Manager”, an application to manage installations for Node.js.

{% tabs %}
{% tab title="Windows" %}
For Windows you can use install [nvm-windows](https://github.com/coreybutler/nvm-windows), by downloading and running their installation binary optimised for Windows machines.

[Download latest release of nvm-windows](https://github.com/coreybutler/nvm-windows/releases/download/1.1.7/nvm-setup.zip)
{% endtab %}

{% tab title="macOS/Linux" %}
For macOS/Linux system you can use install [nvm](https://github.com/nvm-sh/nvm) by running the following command in your Terminal application:

```bash
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.35.3/install.sh | bash
```

Make sure to restart the terminal after running the system.
{% endtab %}
{% endtabs %}

To ensure your installation was successful, open your Terminal or Powershell application and run the following command:

```bash
nvm install 12.9.1
```

You will be shown a few installation instructions, followed by a command similar to the following `Now using node v12.9.1 (npm v6.10.2)`. To see if the installation was successful, run now the next command within the same application you had before:

```bash
node -v
```

If your screen shows `v12.9.1` you are ready to go to the next step.

{% hint style="info" %}
 In case any of the steps did not work for you, go to each of the [nvm](https://github.com/nvm-sh/nvm) or [nvm-windows](https://github.com/coreybutler/nvm-windows) links to troubleshoot the installation.
{% endhint %}

## Step 2 - Download HOPR Chat

Go to our [GitHub Releases Page](https://github.com/hoprnet/hopr-core/releases) and download the latest version, depending to your ecosystem.

* [Download the latest Windows release.](https://github.com/hoprnet/hopr-core/releases/download/1.1.6-dev.c992f25/hopr-chat-nodebin-windows.zip)
* [Download the latest macOS release.](https://github.com/hoprnet/hopr-core/releases/download/1.1.6-dev.c992f25/hopr-chat-nodebin-macos.zip)
* [Download the latest Linux release.](https://github.com/hoprnet/hopr-core/releases/download/1.1.6-dev.c992f25/hopr-chat-nodebin-linux.zip)

## Step 3 - Run HOPR Chat

After downloading the zip file, unzip the contents of the distributed binary and click on the execution file depending on your operating system.

{% tabs %}
{% tab title="Windows" %}
For Windows, double-click on the file named `hopr-chat` with `.bat` extension and described as `Windows Batch File` on its Type attribute.

![HOPR Chat executable for Windows Binary](../../.gitbook/assets/image%20%289%29.png)
{% endtab %}

{% tab title="macOS" %}
For macOS, double-click on the file named `hopr-chat.command` with `.command` extension and described as `Terminal shell script` on its Kind attribute.

![HOPR Chat Executable for macOS](../../.gitbook/assets/image%20%2812%29.png)
{% endtab %}

{% tab title="Linux" %}
For Linux based systems, execute the file named `hopr-chat.sh` with `.sh` extension and provided with executable permissions on your directory.

![HOPR Chat Executable for Linux](../../.gitbook/assets/image%20%2811%29.png)
{% endtab %}
{% endtabs %}

Running these commands will show you a similar screen as the following one. Congratulations! You are now running **HOPR Chat**.

![HOPR Chat Running](../../.gitbook/assets/hopr-chat-demo.gif)

{% hint style="info" %}
Share your address with the `myAddress` command to other people and send messages with `send $ADDRESS`. You can find other people connected using `crawl`, although sometimes it might take a bit to load. Use the `tab` to autocomplete addresses and commands, and quit the application running `quit`.
{% endhint %}

