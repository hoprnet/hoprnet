---
description: Step and step guide to quickly start using HOPR Chat
---

# Quickstart

The following quick start will help you to quickly install **HOPR Chat** and its dependencies so you can quickly connect to the **HOPR Network.** Please follow the step-by-step instructions to ensure everything works properly.

## Step 1 - Install Node.js

Node.js is JavaScript runtime built on Google Chrome's V8 JavaScript engine, which powers many modern web applications. Node.js allows users to run web applications with the same technology Browsers use to run webpages, but from your local computer. Since **HOPR Chat** runs using JavaScript, we first need to download and install Node.js in your machine to use it.

### Download Node.js

Go to the official [Node.js website](https://nodejs.org/en/) and download version `12.18.2 LTS`.

{% tabs %}
{% tab title="Windows" %}
* [Click here to download Node.js version 12.18.2 LTS for Windows \(64 bits\)](https://nodejs.org/dist/v12.18.2/node-v12.18.2-x64.msi)
* [Click here to download Node.js version 12.18.2 LTS for Windows \(32 bits\)](https://nodejs.org/dist/v12.18.2/node-v12.18.2-x86.msi)
{% endtab %}

{% tab title="MacOS" %}
* [Click here to download Node.js version 12.18.2 LTS for macOS](https://nodejs.org/dist/v12.18.2/node-v12.18.2.pkg)
{% endtab %}

{% tab title="Linux" %}
For Linux-based operating systems, please go to the official [Downloads](https://nodejs.org/en/download/) page of Node.js or see the instructions for installing Node.js using a [Package Manager](https://nodejs.org/en/download/package-manager/).
{% endtab %}
{% endtabs %}

### Install Node.js

To install Node.js, double click in the file that you have downloaded to start the install wizard. The wizard setup will guide you through the installation of Node.js in your operating system, and in most cases, just click on “Next” to all the options given, similar to the image shown.

![](../../.gitbook/assets/windows_install_nodejs.webp)

### Test Node.js 

To test if Node.js was successfully installed in your computer, we will run a simple command with Node.js which will output the version. To do so, we will use your Operating System default  _command line interpreter application_ \(“CLI”\) to run a command to see Node.js version.

#### Opening interactive prompt application \(i.e. Terminal or Powershell\)

{% tabs %}
{% tab title="Windows" %}
First open your `Powershell` application. To do so, you need to

1. Click on the `Search Bar` at the bottom of the `Windows Menu`.
2. Type “Powershell” until the app `Windows Powershell` shows up.
3. Press `Enter` or click to open it. A prompt for you to type will show up.

![](../../.gitbook/assets/powershell_open.webp)
{% endtab %}

{% tab title="MacOS" %}
First open your `Terminal` application. To do so, you need to

1. Press `CMD + Space` to open the `Spotlight Search` option.
2. Type “Terminal” until the app `Terminal` shows up.
3. Press `Enter` to open it. A prompt for you to type will show up.

![](../../.gitbook/assets/terminal_open.webp)
{% endtab %}

{% tab title="Linux" %}
First open your `Terminal` application. To do so, you need to open your Task Manager or Applications Menu and look for `Terminal`. The application might look different depending on your Operating System and/or Linux version.
{% endtab %}
{% endtabs %}

#### Running node version command

After your CLI is open, please type the following command in the prompt.

```bash
node -v
```

If your screen shows `v12.18.2` you are ready to go!

## Step 2 - Download HOPR Chat

With Node.js successfully installed in our system, we are ready to download **HOPR Chat**. 

### Download HOPR Chat

Go to our [GitHub Releases Page](https://github.com/hoprnet/hopr-core/releases) and download the latest version.

{% tabs %}
{% tab title="Windows" %}
* [Download the latest Windows release.](https://github.com/hoprnet/hopr-core/releases/download/1.1.6-dev.c992f25/hopr-chat-nodebin-windows.zip)
{% endtab %}

{% tab title="MacOS" %}
* [Download the latest macOS release.](https://github.com/hoprnet/hopr-core/releases/download/1.1.6-dev.c992f25/hopr-chat-nodebin-macos.zip)
{% endtab %}

{% tab title="Linux" %}
* [Download the latest Linux release.](https://github.com/hoprnet/hopr-core/releases/download/1.1.6-dev.c992f25/hopr-chat-nodebin-linux.zip)
{% endtab %}
{% endtabs %}

### Extracting HOPR Chat

Right now, **HOPR Chat** is distributed as a zip file, so you will need to “unzip” its contents first. In some operating systems \(e.g. macOS\), you can just double click on the zip file to do so. For Windows, select the option “Extract All” in the File Explorer to extract **HOPR Chat** files.

![](../../.gitbook/assets/downloading_hopr_bin.webp)

## Step 3 - Run HOPR Chat

Running **HOPR Chat** depends on your Operating System \(OS\). We are distributing different files depending on the OS you are running. Please read the next instructions to know which file to click.

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

## Step 4 - Exploring HOPR Chat

With **HOPR Chat** up and running, you are now ready to type and communicate with other users of the **HOPR Network.** Use any of the following commands to learn more about **HOPR Chat** and how you can use to send messages to other users.

{% tabs %}
{% tab title="myAddress" %}
Share your address with the `myAddress` command to other people. Your **HOPR Address**  starts with `16Uiu2` and it’s the only thing other individuals need to know to send you messages. Use a separate channel to share your **HOPR Address** with your friends.
{% endtab %}

{% tab title="send" %}
You can send messages to other users with the `send` command. To use it, write “send” followed by a **HOPR Address** you want to send a message to. So for instance, to send a message to `16Uiu2HAm62VfBkydtQVtKMUaNC3Upe7rYehGu3eLjFAqrxX1vxsx`, you would need to type `send 16Uiu2HAm62VfBkydtQVtKMUaNC3Upe7rYehGu3eLjFAqrxX1vxsx`. As a pro tip you can first write `send`, and then press the “Tab” key to see who you can send a message to.
{% endtab %}

{% tab title="crawl" %}
You can find other people connected using `crawl`, although sometimes it might take a bit to load. Bear in mind sometimes disconnected addresses will show-up during crawl.
{% endtab %}

{% tab title="help" %}
If you want to see more information about **HOPR Chat** and other additional commands available, you can use the `help` command to give you an overview of the application.
{% endtab %}

{% tab title="quit" %}
As its name suggests, `quit` will exit the **HOPR Chat** application.
{% endtab %}
{% endtabs %}

Inside **HOPR Chat** you can press the “Tab” key to autocomplete addresses and commands. For more detailed instructions, go to the next page, where you can find more specific instructions on how to get started with the Advanced Setup.

