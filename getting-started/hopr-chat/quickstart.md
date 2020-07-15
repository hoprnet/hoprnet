---
description: Step and step guide to quickly start using HOPR Chat
---

# Quickstart

### Before Getting Started

This quickstart guide will help you to quickly install **HOPR Chat** and its dependencies so you can quickly connect to the **HOPR Network.** Please follow the step-by-step instructions to ensure everything works properly. As instructions might be different depending on your Operating System \(e.g. Windows, MacOS, Linux\), our instructions will be shown under “Tabs” like the following:

{% tabs %}
{% tab title="Windows" %}
In some cases, you will need to run commands and instructions as an Administrator. Ensure you have Administrator access, and a working Internet connection. Firewall prompts might show up, which only requires you to accept them on request.
{% endtab %}

{% tab title="MacOS" %}
For MacOS, we will sometimes give you Keyboard shortcuts to help you navigate the system. We will describe these shortcuts with a combination of keys such as `⌘c` to described the `Command` key followed by the `c` key \(a common shortcut for copying content\). For instance, in this guide we'll make use of the `Spotlight Search`, which you can quickly access by pressing `⌘Space`, which will allow you to quickly look for applications.
{% endtab %}

{% tab title="Linux" %}
All commands for Linux users will be assumed to be executed in the `Terminal` of your Linux distribution. As access to your `Terminal` might vary depending on your distribution, please make sure beforehand you know how to access your `Terminal` before continuing this tutorial.
{% endtab %}
{% endtabs %}

## Step 1 - Install Node.js

Node.js is a JavaScript runtime built on Google Chrome's V8 JavaScript engine, which powers many modern web applications. Node.js allows users to run web applications with the same technology browsers use to run webpages, but from your local computer. Since **HOPR Chat** runs using JavaScript, we first need to download and install Node.js on your machine to use it.

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

To install Node.js, double-click the file you just downloaded to start the install wizard. The wizard will guide you installing Node.js on your operating system. In most cases, you'll just want to click “Next” for all the options given, similar to the image shown.

![](../../.gitbook/assets/windows_install_nodejs.webp)

### Test Node.js 

To check that Node.js was successfully installed, we will run a simple command with Node.js which will output the version number. To do this, we will use your operating system's default  _command line interpreter application_ \(“CLI”\) to run a command to see Node.js version.

#### Opening interactive prompt application \(Terminal or Powershell\)

{% tabs %}
{% tab title="Windows" %}
First open your `Powershell` application. To do this, you'll need to:

1. Click the `Search Bar` at the bottom of the `Windows Menu`.
2. Type “Powershell” until the app `Windows Powershell` shows up.
3. Press `Enter` or click to open it. A prompt for you to type will show up.

![](../../.gitbook/assets/powershell_open.webp)
{% endtab %}

{% tab title="MacOS" %}
First open your `Terminal` application. To do this, you'll need to:

1. Press `CMD + Space` to open the `Spotlight Search` option.
2. Type “Terminal” until the app `Terminal` shows up.
3. Press `Enter` to open it. A prompt for you to type will show up.

![](../../.gitbook/assets/terminal_open.webp)
{% endtab %}

{% tab title="Linux" %}
First open your `Terminal` application. To do this, you'll need to open your task manager or applications menu and look for `Terminal`. The application might look different depending on your operating system and/or Linux version.
{% endtab %}
{% endtabs %}

#### Running node version command

After your CLI is open, please type the following command in the prompt and press `Enter`.

```bash
node -v
```

If your screen shows `v12.18.2` you are ready to go!

## Step 2 - Download HOPR Chat

With Node.js successfully installed on your system, you're ready to download **HOPR Chat**. 

### Download HOPR Chat

Go to our [GitHub Releases Page](https://github.com/hoprnet/hopr-core/releases) and download the latest version.

{% tabs %}
{% tab title="Windows" %}
* [Download the latest Windows release.](https://github.com/hoprnet/hopr-core/releases/download/1.1.6-testnet.c7aea14/hopr-chat-nodebin-windows.zip)
{% endtab %}

{% tab title="MacOS" %}
* [Download the latest macOS release.](https://github.com/hoprnet/hopr-core/releases/download/1.1.6-testnet.c7aea14/hopr-chat-nodebin-macos.zip)
{% endtab %}

{% tab title="Linux" %}
* [Download the latest Linux release.](https://github.com/hoprnet/hopr-core/releases/download/1.1.6-testnet.c7aea14/hopr-chat-nodebin-linux.zip)
{% endtab %}
{% endtabs %}

### Extracting HOPR Chat

Right now, **HOPR Chat** is distributed as a .zip file, so you will need to “unzip” its contents first. In some operating systems \(e.g., macOS\), you can just double click on the .zip file to do this. For Windows, select the option “Extract All” in the File Explorer to extract **HOPR Chat** files.

![](../../.gitbook/assets/downloading_hopr_bin.webp)

## Step 3 - Run HOPR Chat

How you run **HOPR Chat** depends on your operating system \(OS\). We are distributing different files depending on the OS you are running. Please read the next instructions to know which file to click.

{% tabs %}
{% tab title="Windows" %}
For Windows, double-click  the file named `hopr-chat` with `.bat` extension and described as `Windows Batch File` in its Type attribute.

![HOPR Chat executable for Windows Binary](../../.gitbook/assets/image%20%289%29.png)
{% endtab %}

{% tab title="MacOS" %}
For macOS, double-click on the file named `hopr-chat.command` with `.command` extension and described as `Terminal shell script` in its Kind attribute.

![HOPR Chat Executable for macOS](../../.gitbook/assets/image%20%2812%29.png)

When opening the `hopr-chat.command` file, you will be prompted by `Gatekeeper`, the security mechanism from macos, that the application has been downloaded by a non-recognised URL. This is a security mecanism activated by default in macos.

To go ahead, you will need to tell `Gatekeeper` that the application is to be trusted. To do so, open `Security & Privacy` in your `System Preferences` and click `Open Anyway`under the `General` tab. This will prompt `Gatekeeper` to ask you again to open **HOPR Chat** with an option to actually run it. Select `Open` and you will see **HOPR Chat** in your screen.

![](../../.gitbook/assets/hopr_macos_gatekeeper.webp)
{% endtab %}

{% tab title="Linux" %}
For Linux-based systems, execute the file named `hopr-chat.sh` with `.sh` extension and provided with executable permissions on your directory.

![HOPR Chat Executable for Linux](../../.gitbook/assets/image%20%2811%29.png)
{% endtab %}
{% endtabs %}

Running these commands will bring up a screen similar the one shown below. Congratulations! You are now running **HOPR Chat**.

![HOPR Chat Running](../../.gitbook/assets/hopr-chat-demo.gif)

## Step 4 - Exploring HOPR Chat

With **HOPR Chat** up and running, you are now ready to type and communicate with other users on the **HOPR Network.** Use any of the following commands to learn more about **HOPR Chat** and how to send messages to other users.

{% tabs %}
{% tab title="myAddress" %}
Share your address with the `myAddress` command to other people. Your **HOPR Address**  starts with `16Uiu2` and it’s the only thing other people need to know to send you messages. Use a separate channel to share your **HOPR Address** with your friends.
{% endtab %}

{% tab title="send" %}
You can send messages to other users with the `send` command. To use it, write “send” followed by a **HOPR Address** you want to send a message to. So for instance, to send a message to `16Uiu2HAm62VfBkydtQVtKMUaNC3Upe7rYehGu3eLjFAqrxX1vxsx`, you would need to type `send 16Uiu2HAm62VfBkydtQVtKMUaNC3Upe7rYehGu3eLjFAqrxX1vxsx`.   
  
You can also write `send`, and then press the “Tab” key to see who you can send a message to.
{% endtab %}

{% tab title="crawl" %}
You can find other people connected using `crawl`, although sometimes it might take a while to complete. Bear in mind that sometimes disconnected addresses will show up during crawl.
{% endtab %}

{% tab title="help" %}
If you want to see more information about **HOPR Chat** and other additional commands available, you can use the `help` command to give you an overview of the application.
{% endtab %}

{% tab title="quit" %}
Typing `quit` will exit the **HOPR Chat** application.
{% endtab %}
{% endtabs %}

Inside **HOPR Chat** you can press the “Tab” key to autocomplete addresses and commands. For more detailed instructions, go to the next page, where you can find more specific instructions on how to get started with the Advanced Setup.

