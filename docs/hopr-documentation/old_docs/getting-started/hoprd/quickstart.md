<!-- ---
description: Step and step guide to quickly start using HOPR Chat
--- -->

# Quickstart

### Before Getting Started

This quick-start guide will help you to quickly install **HOPR Chat** and its dependencies so you can quickly connect to the **HOPR Network.** Please follow the step-by-step instructions to ensure everything works.

{% hint style="warning" %}
**HOPR**, **HOPR Chat** and the **HOPR Network** are early-stage technologies. The currently available early test versions are meant for the brave explorers who are here to build the future of a more private Web3 together with us. Do not rely on HOPR to protect your privacy or assets just yet!
{% endhint %}

As instructions might be different depending on your Operating System \(e.g. Windows, MacOS, Linux\), our instructions will be shown under “Tabs” like the following ones. Please select your Operating System before continue reading the guide.

{% tabs %}
{% tab title="Windows" %}
In some cases, you will need to run commands and instructions as an Administrator. Ensure you have Administrator access and a working Internet connection. Firewall prompts might show up, which only requires you to accept them on request.
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

Click the following link to download Node.js version `12.18.2 LTS` in your computer:

{% tabs %}
{% tab title="Windows" %}
[Click here to download Node.js version 12.18.2 LTS for Windows \(64 bits\)](https://nodejs.org/dist/v12.18.2/node-v12.18.2-x64.msi)
{% endtab %}

{% tab title="MacOS" %}
[Click here to download Node.js version 12.18.2 LTS for macOS](https://nodejs.org/dist/v12.18.2/node-v12.18.2.pkg)
{% endtab %}

{% tab title="Linux" %}
For Linux-based operating systems, please go to the official [Downloads](https://nodejs.org/en/download/) page of Node.js or see the instructions for installing Node.js using a [Package Manager](https://nodejs.org/en/download/package-manager/).
{% endtab %}
{% endtabs %}

{% hint style="info" %}
You can also go to the official Node.js website to [download](https://nodejs.org/en/download/) the installer.
{% endhint %}

### Install Node.js

To install Node.js, double-click the file you just downloaded to start the install wizard. The wizard will guide you installing Node.js on your operating system. In most cases, you'll just want to click “Next” for all the options given, similar to the image shown.

![](../../images/windows_install_nodejs.webp)

### Test Node.js

To check that Node.js was successfully installed, we will run a simple command with Node.js which will output the version number. To do this, we will use your Operating System's `Terminal` to run a command to see the Node.js version.

#### Opening Terminal

{% tabs %}
{% tab title="Windows" %}
In Windows, we'll open the `Powershell` application. To do this, you'll need to:

1. Click the `Search Bar` at the bottom of the `Windows Menu`.
2. Type “Powershell” until the app `Windows Powershell` shows up.
3. Press `Enter` or click to open it. A prompt for you to type will show up.

![](../../images/powershell_open.webp)
{% endtab %}

{% tab title="MacOS" %}
Follow the next instructions to open the `Terminal` application:

1. Press `⌘Space` to open the `Spotlight Search` option.
2. Type “Terminal” until the app `Terminal` shows up.
3. Press `Enter` to open it. A prompt for you to type will show up.

![](../../images/terminal_open.webp)
{% endtab %}

{% tab title="Linux" %}
The `Terminal` application might look different depending on your operating system and/or Linux version. Please Google for the specific instructions according to your distribution.
{% endtab %}
{% endtabs %}

#### Running Node Version Command

After your `Terminal` or `Powershell` is open, please type `node -v` in the prompt and press `Enter`. Ensure there's an empty space between the words `node` and `-v`

![Type “node -v” to find your node version. It should show v12.18.2](../../images/node-version.gif)

If your screen shows `v12.18.2` you are ready to go!

## Step 2 - Get HOPR Chat

With Node.js successfully installed on your system, you're ready to download **HOPR Chat,** which is distributed as a zip file.

### Download HOPR Chat

Download the latest version of **HOPR Chat** by clicking in the following link.

{% tabs %}
{% tab title="Windows" %}
[Download the latest Windows release.](https://github.com/hoprnet/hopr-chat/releases/download/v1.3.0/hopr-chat-nodebin-windows.zip)
{% endtab %}

{% tab title="MacOS" %}
[Download the latest macOS release.](https://github.com/hoprnet/hopr-chat/releases/download/v1.3.0/hopr-chat-nodebin-macos.zip)
{% endtab %}

{% tab title="Linux" %}
[Download the latest Linux release.](https://github.com/hoprnet/hopr-chat/releases/download/v1.3.0/hopr-chat-nodebin-linux.zip)
{% endtab %}
{% endtabs %}

{% hint style="info" %}
You can see all our releases in our GitHub [releases](https://github.com/hoprnet/hopr-core/releases) page.
{% endhint %}

### Extracting HOPR Chat

Right now, **HOPR Chat** is distributed as a .zip file, so you will need to “unzip” its contents first.

{% tabs %}
{% tab title="Windows" %}
For Windows, select the option “Extract All” in the File Explorer to extract **HOPR Chat** files.

![](../../images/downloading_hopr_bin.webp)
{% endtab %}

{% tab title="MacOS" %}
For MacOS, just double click on the `.zip` file you downloaded to see the contents of the **HOPR Chat** application.

![HOPR Chat contents of the extracted content should look as follows.](../../images/hopr-macos-contents.webp)

You can also do this from your Browser by just clicking the tab that shows up the downloaded folder.
{% endtab %}

{% tab title="Linux" %}
In Linux, you need to use `unzip` or similar utility to extract the contents to run **HOPR Chat.** Use your Linux distribution package manager to install and afterwards run in your `Terminal` application the following command in the directory you downloaded **HOPR Chat.**

```bash
$ unzip hopr-chat-nodebin-linux.zip
```

The contents fo the extracted folder should look as follows:

![HOPR Chat contents extracted as seen in a Linux terminal using the ls command](../../images/hopr-linux-contents.webp)
{% endtab %}
{% endtabs %}

## Step 3 - Run HOPR Chat

**HOPR Chat** is an interactive chat application. To run, you need to click on the executable you previously extracted in the last step.

{% tabs %}
{% tab title="Windows" %}
For Windows, double-click the file named `start-hopr-chat` with `.bat` extension and described as `Windows Batch File` in its Type attribute.

![HOPR Chat executable file for Windows.](../../images/image.png)
{% endtab %}

{% tab title="MacOS" %}
For macOS, double-click on the file named `start-hopr-chat.command` with `.command` extension and described as `Terminal shell script` in its Kind attribute.

![HOPR Chat executable as seen in MacOS](../../images/hopr-macos-contents.webp)

When opening the `start-hopr-chat.command` file, you will be prompted by `Gatekeeper`, the security mechanism from macOS, that the application has been downloaded by a non-recognised URL. This is a security mechanism activated by default in macOS.

To go ahead, you will need to tell `Gatekeeper` that the application is to be trusted. To do so, open `Security & Privacy` in your `System Preferences` and click `Open Anyway`under the `General` tab. This will prompt `Gatekeeper` to ask you again to open **HOPR Chat** with an option to actually run it. Select `Open` and you will see **HOPR Chat** in your screen.

![](../../images/hopr_macos_gatekeeper.webp)
{% endtab %}

{% tab title="Linux" %}
For Linux-based systems, execute the file named `start-hopr-chat.sh` with `.sh` extension and provided with executable permissions on your directory.

![HOPR Chat executable as seen in a Linux terminal](../../images/hopr-linux-contents.webp)
{% endtab %}
{% endtabs %}

Running these commands will bring up a screen similar the one shown below. Congratulations! You are now running **HOPR Chat**.

![HOPR Chat Testnet up and running.](../../images/hopr-chat-testnet.gif)

## Step 4 - Get Your HOPR Address

With **HOPR Chat** up and running, you are now ready to type and communicate with other users on the **HOPR Network.** To do so, you need to get your **HOPR Address.** To do so, write `address` and press `Enter` inside **HOPR Chat.** Something like the following will show up:

```text
> address
ethereum:  0x9e95cdcb480f133b0c1af70613d1488ee01bf53e
HOPR:      16Uiu2HAm34oK6EyA2SuFo9rHXpm5Kwy6C8MeJ26JaRFBzgdQqVpZ
```

Share your address shown by the `address` command to other people. Your HOPR Address starts with `16Uiu2` and it’s the only thing other people need to know to send you messages. Use a separate channel to share your HOPR Address with your friends.

## Step 5 - Send Messages to Other HOPR Nodes

You can send a message to other users with the `send` command, to send multiple messages, just repeat the following steps multiple times.

To send a message, type `send` followed by a **HOPR Address** you want to send a message to. So for instance, to send a message to `16Uiu2HAm62VfBkydtQVtKMUaNC3Upe7rYehGu3eLjFAqrxX1vxsx`, you would need to type `send 16Uiu2HAm62VfBkydtQVtKMUaNC3Upe7rYehGu3eLjFAqrxX1vxsx` followed by pressing `Enter`. Afterwards, you can type whatever message you want to send to that **HOPR Node;** after you have completed your message, send it by hitting `Enter`.

{% hint style="info" %}
To send a message to another **HOPR Node,** you first need another individual‘s **HOPR Address** to send a message. You can jump to our Telegram to ask other HOPR users to share their **HOPR Address**. You can see available connected nodes by pressing the key “Tab” after typing `send`, which also functions as an auto-complete \(i.e. shows you possible addresses every time you press the “Tab” key\). For questions, please reach us via [Telegram](https://t.me/hoprnet).
{% endhint %}

Assuming someone gives you the address `16Uiu2HAmVnjSZeEwKvWxGS4cbZyNgnTzXzU3tRQNAeapJFgeyoBR`, sending a message to them would require you to first write `send`, pasting the the address \(either pressing `Ctrl+V` in Windows or `⌘v` in MacOS\) and then press `Enter`. This would look as follows:

```text
> send 16Uiu2HAmVnjSZeEwKvWxGS4cbZyNgnTzXzU3tRQNAeapJFgeyoBR
Type in your message and press ENTER to send:
```

After typing your message, make sure to press `Enter` again. Your message will be sent anonymously to the recipient of the **HOPR Address** and disappear from your screen. A prompt like the following will show:

```text
Sending message to 16Uiu2HAmVnjSZeEwKvWxGS4cbZyNgnTzXzU3tRQNAeapJFgeyoBR ...
---------- New Packet ----------
Destination    : 16Uiu2HAmVnjSZeEwKvWxGS4cbZyNgnTzXzU3tRQNAeapJFgeyoBR
--------------------------------
```

The recipient of the **HOPR Address** will receive something like the following:

```text
===== New message ======
Message: Welcome to the HOPR Network.
Latency: 23577 ms
========================
```

#### Congratulations! You have successfully sent a message to the **HOPR Network**. Your message has been relayed and packaged securely and privately across the internet.

## Next Steps

To learn more about our network, please go to our **Core Concepts** section. You can also learn how to connect to a different network by going to our **Advanced Setup.** For more information and updates about the **HOPR Network**, please follow our [Twitter](https://twitter.com/hoprnet). For questions and additional information, please go to our [Telegram](https://t.me/hoprnet) channel.

If you do not want to connect to the **HOPR Network** anymore, you can just close the terminal window in which you have **HOPR Chat** running. To **uninstall** just delete the `HOPR Chat` folder you downloaded.
