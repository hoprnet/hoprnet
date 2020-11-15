---
description: Learn the advanced setup information for using HOPR Chat on your system
---

# Advanced Setup

This setup guide will show you how to use **HOPR** by installing **HOPR Chat** on your system using Docker or Node.js. In this step-by-step guide, we will download the necessary software, run **HOPR Chat,** and send a message to another user in the **HOPR network,** in the same way the developers of **HOPR Chat** do. For a faster quick start, use our Quickstart guide.

{% page-ref page="quickstart.md" %}

Depending on your device capabilities, pick the method that will work best for you. Using Docker is faster than using Node.js, but has higher hardware and disk space requirements, as well as some virtualization settings enabled. On the other hand, Node.js requires a bit more setup, but can be completed by even limited devices.

{% hint style="danger" %}
As changes are continuous in our codebase, please bear in mind some of this documentation might be outdated and errors might occur. Our [Telegram](https://t.me/hoprnet) channel is currently the best channel to be up to date with the latest developments of the **HOPR Chat**, **HOPR** and the **HOPR Network.**
{% endhint %}

## Setting up your machine for HOPR Chat

### Using Docker

Using Docker allows you quickly get started with **HOPR Chat** without having to download any other software. However, there are hardware requirements you should be aware of.

To use Docker, you will need a device that supports hardware-level virtualisation: VT-x for Intel-based PCs and AMD-V for AMD processors. Most Mac and Linux machines support this out of the box, but for PC you will need to make sure it's turned on and perhaps enable it in the BIOS settings. It will be different for different BIOS, just look for VT-x / AMD-V switch.

Additionally, ensure you have enough memory \(approx. 2 GB\) and disk space \(approx. 1 GB\) before starting.

#### How to Install Docker

Before doing anything else, you need to install [Docker Desktop](https://hub.docker.com/editions/community/docker-ce-desktop-mac/) on your machine. Docker is natively supported in MacOS/Linux, and will prompt you with any additional requirements, depending on your operating system. Windows 10 will require different configurations depending on your version \(Home or Pro/Enterprise\). Depending on your setup, you might need to follow additional steps to ensure your computer works properly with Docker.

{% tabs %}

Depending of your distribution, please follow the official guidelines for how to install and run Docker on your workstation.

* [Installing Docker in Ubuntu](https://docs.docker.com/engine/install/ubuntu/)
* [Installing Docker in Fedora](https://docs.docker.com/engine/install/fedora/)
* [Installing Docker in Debian](https://docs.docker.com/engine/install/debian/)
* [Installing Docker in CentOS](https://docs.docker.com/engine/install/centos/)
* Visit [Docker Hub ](https://hub.docker.com/editions/community/docker-ce-desktop-mac/)and download **Docker Desktop** to your computer.
* Follow the wizard steps to ensure Docker is installed.
* Ensure the installation was successful by running `docker ps` in your terminal.
* Go to [Docker Hub](https://docs.docker.com/toolbox/overview/) to download **Docker Toolbox** to your computer.
* Follow-up the wizard steps to ensure Docker is installed.
* Ensure the installation was successful by running `docker ps`
* Go to [Docker ](https://www.docker.com/products/docker-desktop)and download **Docker Desktop** to your computer.
* Follow-up the wizard steps to ensure Docker is installed.
* Ensure the installation was successful by running `docker ps`

#### Downloading HOPR Chat image from Docker Hub

Once Docker is up and running, you need to download a valid **HOPR Chat** Docker Image. To do so, run `docker pull hopr/chat` from your terminal. This process may take some time depending on your internet connection, as it will download around `0.5 GB` from our [Docker Hub Registry](https://hub.docker.com/r/hopr/chat).

![Currently HOPR Chat is about ~0.5 GB, please be patient.](../.gitbook/assets/docker_install_macos%20%283%29%20%281%29%20%281%29%20%281%29%20%281%29%20%281%29%20%281%29.gif)

To ensure your machine has successfully downloaded **HOPR Chat,** run `docker images`. You will be shown the **HOPR Chat** image being installed locally, ready to be run.

![HOPR Chat distributed as a Docker image](../.gitbook/assets/docker_images%20%283%29%20%281%29%20%281%29%20%281%29%20%281%29%20%281%29%20%281%29.gif)

{% hint style="info" %}
Docker images can quickly go out of date. We recommend reviewing which are the latest images available to be used before downloading one. You can see all our available images and their publication date in our public [Docker registry](https://gcr.io/hoprassociation/hopr-chat).
{% endhint %}

### Using Node.js

Using Node.js allows you to run **HOPR Chat** as a Node.js application, ensuring your experience is as close as possible to the one the developers had when developing **HOPR Chat** and the **HOPR Core** protocol. Node.js might require further software installation, but is can be run on machines with lower hardware specifications. It also takes up considerably less space than Docker \(approx. 50MB vs approx. 1GB\).

#### How to Install Node.js using nvm

To use Node.js, we recommend installing [nvm](https://github.com/nvm-sh/nvm), a Node.js version manager. This ensures we can install and uninstall as many versions of Node.js as needed. It will also help if you have any additional installation requirements for running Node.js. If you're using Windows, you will need [nvm-windows](https://github.com/coreybutler/nvm-windows), which is its PC equivalent.

To install nvm on Linux or macOS, please follow the instructions on their [GitHub website](https://github.com/nvm-sh/nvm), or run any of the following commands in your terminal instead \(we will use nvm `v0.35.3`\) depending on whether you have `curl` or `wget` in your system.

{% tabs %}
{% tab title="cURL" %}
```text
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.35.3/install.sh | bash
```
{% endtab %}

{% tab title="Wget" %}
```text
wget -qO- https://raw.githubusercontent.com/nvm-sh/nvm/v0.35.3/install.sh | bash
```
{% endtab %}
{% endtabs %}

For Windows, download the nvm-windows latest binary, available on their [releases](https://github.com/coreybutler/nvm-windows/releases) page.

**Installing Node.js**

Once you have downloaded and set up nvm on your machine \(run `nvm ls` to ensure everything is in place\), you will need to install a specific version of Node.js before running **HOPR Chat**.

At the time of writing, **HOPR Chat** runs on Nodejs `>v12`. Specifically, **HOPR Chat** has been developed and tested in `v12.9.1` \(so if you run into any issues with **HOPR Chat,** try switching to `v12.9.1` to see if they disappear\).

To install Node.js with nvm, run the following in your Terminal, Bash or Powershell.

```text
$ nvm install v12.9.1
$ nvm use v12.9.1
```

If everything was done properly, you can run `node --version` to see your current `node` version, alongside running basic commands as shown when running simply `node` in your terminal.

![](../.gitbook/assets/node%20%284%29%20%281%29%20%281%29%20%281%29%20%281%29%20%281%29%20%281%29.gif)

## **Running HOPR Chat**

### **Using Docker**

To run **HOPR Chat** via Docker**,** you need to copy and paste the following command. You can replace `switzerland` with anything else, but make sure you always use the same password as this will be used by **HOPR Chat**. You can also use your own [Infura](https://infura.io/) credentials instead of the ones provided here, but ensure you use the Kovan provider, as the **HOPR Core** Ethereum contracts are currently only deployed there.

#### macOS/Linux commands

{% tabs %}
{% tab title="ch-t-01" %}
```text
docker run -v $(pwd)/db:/app/db \
-e HOST_IPV4=0.0.0.0:9091 \
-e BOOTSTRAP_SERVERS=/ip4/34.65.219.148/tcp/9091/p2p/16Uiu2HAkwSEiK819yvnG84pNFsqXkpFX4uiCaNSwADnmYeAfctRn \
-e ETHEREUM_PROVIDER=wss://kovan.infura.io/ws/v3/f7240372c1b442a6885ce9bb825ebc36 \
-p 9091:9091 -it hopr/chat -p switzerland
```
{% endtab %}

{% tab title="ch-t-02" %}
```text
docker run -v $(pwd)/db:/app/db \
-e HOST_IPV4=0.0.0.0:9091 \
-e BOOTSTRAP_SERVERS=/ip4/34.65.148.229/tcp/9091/p2p/16Uiu2HAmRsp3VBLcyPfTBkJYEwS47bewxWqqm4sEpJEtPBLeV93n \
-e ETHEREUM_PROVIDER=wss://kovan.infura.io/ws/v3/f7240372c1b442a6885ce9bb825ebc36 \
-p 9091:9091 -it hopr/chat -p switzerland
```
{% endtab %}
{% endtabs %}

#### Windows commands

{% tabs %}
{% tab title="ch-t-01" %}
```text
docker run -v %cd%/db:/app/db ^
-e HOST_IPV4=0.0.0.0:9091 ^
-e BOOTSTRAP_SERVERS=/ip4/34.65.219.148/tcp/9091/p2p/16Uiu2HAkwSEiK819yvnG84pNFsqXkpFX4uiCaNSwADnmYeAfctRn ^
-e ETHEREUM_PROVIDER=wss://kovan.infura.io/ws/v3/f7240372c1b442a6885ce9bb825ebc36 ^
-p 9091:9091 -it hopr/chat -p switzerland
```
{% endtab %}

{% tab title="ch-t-02" %}
```text
docker run -v %cd%/db:/app/db ^
-e HOST_IPV4=0.0.0.0:9091 ^
-e BOOTSTRAP_SERVERS=/ip4/34.65.148.229/tcp/9091/p2p/16Uiu2HAmRsp3VBLcyPfTBkJYEwS47bewxWqqm4sEpJEtPBLeV93n ^
-e ETHEREUM_PROVIDER=wss://kovan.infura.io/ws/v3/f7240372c1b442a6885ce9bb825ebc36 ^
-p 9091:9091 -it hopr/chat -p switzerland
```
{% endtab %}
{% endtabs %}

You will be welcomed by the following message.

![](../.gitbook/assets/hopr%20%283%29%20%281%29%20%281%29%20%281%29%20%281%29%20%281%29%20%281%29.gif)

If you get an `Unable to connect to Bootstrap node` message, use other bootstrap nodes. Our bootstrap nodes are available as `TXT` records behind our `hoprnet.org` domain, so you can see them using the `dig` command in any linux computer.

```text
dig -t TXT _dnsaddr.bootstrap.testnet.hoprnet.org
```

This will return an output with some Bootstrap Nodes address:

```text
;; ANSWER SECTION:
_dnsaddr.bootstrap.testnet.hoprnet.org. 299 IN TXT "dnsaddr=/ip4/34.65.219.148/tcp/9091/p2p/16Uiu2HAkwSEiK819yvnG84pNFsqXkpFX4uiCaNSwADnmYeAfctRn"
_dnsaddr.bootstrap.testnet.hoprnet.org. 299 IN TXT "dnsaddr=/ip4/34.65.148.229/tcp/9091/p2p/16Uiu2HAmRsp3VBLcyPfTBkJYEwS47bewxWqqm4sEpJEtPBLeV93n"
```

You can then replace `BOOTSTRAP_SERVERS` value with `/ip4/34.65.219.148/tcp/9091/p2p/16Uiu2HAkwSEiK819yvnG84pNFsqXkpFX4uiCaNSwADnmYeAfctRn` or both values displayed \(a comma is needed if you put both\).

{% hint style="warning" %}
At the time of writing, Bootstrap Nodes are not able to connect to each other, which means connecting via Bootstrap Node 1 will not allow you to connect to **HOPR Nodes** that use Bootstrap Node 2. Ideally, you use as many Bootstrap Nodes as possible to have a bigger access to the network.

Bootstrap Nodes are maintained by the **HOPR Association,** and can change w/o previous notice during our Testnet Period. You can replace `testnet` for `develop` in the `dig` command to see our Develop Bootstrap Nodes, but those are restarted every week. You can always see the status of our current deployed nodes in [https://status.hoprnet.org/](https://status.hoprnet.org/).
{% endhint %}

You can learn more about Bootstrap Nodes on the page linked below.

{% page-ref page="../core-concepts/bootstrap-nodes.md" %}

After running any of these commands, you will be welcomed by **HOPR Chat**’s introductory screen, which provides you with further instructions on how to send messages to other users connected to the network. Under the wraps, **HOPR Chat** runs also a **HOPR Node**, which listens to port `9091` to incoming connections from other **HOPR Node** users.

{% hint style="info" %}
Depending on your configuration and version of **HOPR Chat**, you might need to fund your **HOPR Chat** account with some tokens. If so, please visit the “**Funding your account**” page below.
{% endhint %}

### Using Node.js

#### Binary Releases

To run **HOPR Chat** via Node.js**,** you can simply download our pre-compiled binary for each version. You can find these binaries in our [Releases](https://github.com/hoprnet/hopr-core/releases) page inside a zip file.

![Please select the correct distribution for your operating system.](../.gitbook/assets/image%20%288%29%20%283%29%20%281%29%20%281%29%20%281%29%20%281%29%20%281%29%20%281%29.png)

These files have **HOPR Chat** pre-configured and compiled to work in your system. Click on the executable file inside that folder to see **HOPR Chat** up and running. Depending on your distribution, this file might be different.

{% tabs %}
{% tab title="Linux" %}
On Linux, double-click or execute the file named `hopr-chat.sh`. Behind the scenes, it will run `node index.js` from the directory you are currently working, ensuring it has the configuration settings distributed with the binary.
{% endtab %}

{% tab title="macOS" %}
On macOS, double-click or execute the file named `hopr-chat.command`. Behind the scenes, it will run `node index.js` from the directory you are currently working, ensuring it has the configuration settings distributed with the binary.
{% endtab %}

{% tab title="Windows 10" %}
On Windows, double-click the file named `hopr-chat.bat` or right-click and select the option “Run with Powershell”. A prompt from Windows Defender might request your permission to run it. Behind the scenes, it will run `node index.js` from the directory you are currently working, ensuring it has the configuration settings distributed with the binary.
{% endtab %}
{% endtabs %}

As soon as you double-click the executable file, you will be welcomed by the **HOPR Chat** initial message, which might look different depending on your OS. Latest releases include a default password, but normally, a password will be required to lock your **HOPR Node**, and every time you run **HOPR Chat** it will prompt you for the password again.

{% hint style="info" %}
Since **HOPR Chat** is being distributed as a Node.js binary, the included pre-compiled binary [might trigger some prompts in macOS](https://docs.hoprnet.io/home/getting-started/hopr-chat/troubleshooting) which you will need to accept and provide access for. To work around these issues, please see our Troubleshooting guide.
{% endhint %}

{% page-ref page="../core-concepts/hopr-chat/troubleshooting.md" %}

#### From source

You can always run **HOPR Chat** from our source code. To do so, you need to clone our [GitHub repository](https://github.com/hoprnet/hopr-chat) with **HOPR Chat** source code.

```text
git clone git@github.com:hoprnet/hopr-chat.git
```

For more information on how to compile and run **HOPR Chat** yourself, please see our project's [README](https://github.com/hoprnet/hopr-chat/blob/master/README.md).

## Sending a HOPR message

With **HOPR Chat** up and running, you can now send messages to any connected nodes in the network. You can either have a friend send you their address, or you can also start another **HOPR Chat** instance. You can also find **HOPR Chat** users in our [Telegram](https://t.me/hoprnet).

{% hint style="info" %}
In case you are using a version of **HOPR Chat** with **HOPR Core** `<v0.6.10`, please ensure you have enough **HOPR Tokens** to send and receive messages.
{% endhint %}

Now, let's find some nodes to talk to. To do this, run `crawl`, which will show you other users that are connected to the **HOPR Network** and are available to chat.

![The crawl command will show you other connected nodes.](../.gitbook/assets/running_hopr_chat_and_crawling%20%283%29%20%281%29%20%281%29%20%281%29%20%281%29%20%281%29%20%281%29.gif)

To talk to other users, copy another connected user address and send a message to them with the `send` command. This will look something like: `send 16Uiu2HAmCtWxx3Ky3ZjtWj1whkezdRvMAYKU9f57CRPj2FkPtWsD`

**HOPR Chat** will then prompt you for a message to send.

![Your message will be sent privately through the HOPR network](../.gitbook/assets/running_hopr_chat_and_sending%20%283%29%20%281%29%20%281%29%20%281%29%20%281%29%20%281%29%20%281%29.gif)

Congratulations! You have communicated with another node using a privacy-preserving decentralised protocol. **HOPR Chat** is just a proof of concept right now, but you can already see the capabilities of the protocol. Click next to learn about **Bootstrap Nodes,** or go back to see the general introduction about **HOPR Chat.**

