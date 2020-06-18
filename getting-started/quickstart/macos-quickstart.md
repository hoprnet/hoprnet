---
description: Get familiar with HOPR on your Mac or Unix based system.
---

# MacOS/Linux Quickstart

This quickstart tutorial will show you how to use **HOPR** by installing **HOPR Chat** on your system using Docker on a MacOS/Linux-based computer. In this step-by-step guide, we will download Docker, run **HOPR Chat,** and send a message to another user connected to the **HOPR network**.

## Step 1 - Installing Docker on your machine

Before doing anything else, you need to install [Docker Desktop](https://hub.docker.com/editions/community/docker-ce-desktop-mac/) on your machine. Docker is natively supported in MacOS/Linux, and will prompt you with any installation requirements, depending on your operating system. Depending on your Linux distribution, you might need to follow additional steps to ensure your computer works properly with Docker.

### Instructions for installing Docker in Linux

Depending of your distribution, please follow the official guidelines for how to install and run Docker on your workstation.

* [Installing Docker in Ubuntu](https://docs.docker.com/engine/install/ubuntu/)
* [Installing Docker in Fedora](https://docs.docker.com/engine/install/fedora/)
* [Installing Docker in Debian](https://docs.docker.com/engine/install/debian/)
* [Installing Docker in CentOS](https://docs.docker.com/engine/install/centos/)

### Instructions for installing Docker in MacOS

1. Visit [Docker Hub ](https://hub.docker.com/editions/community/docker-ce-desktop-mac/)and download Docker Desktop to your computer.
2. Follow the wizard steps to ensure Docker is installed.
3. Ensure the installation was successful by running `docker ps` in your terminal.

## Step 2 - Downloading HOPR Chat image from Docker Hub

To use **HOPR Chat,** run `docker pull hopr/chat` from your terminal. This process may take some time depending on your internet connection.

![Currently HOPR Chat is about ~0.5 GB, please be patient.](../../.gitbook/assets/docker_install_macos.gif)

To ensure your machine has successfully downloaded **HOPR Chat,** run `docker images`.You will be shown the **HOPR Chat** image being installed locally, ready to be run.

![HOPR Chat distributed as a Docker image](../../.gitbook/assets/docker_images.gif)

## Step 3 - **Running HOPR Chat**

To run **HOPR Chat,** you only need to copy and paste the following command. You can replace `switzerland` for anything else, but ensure to always use the same password as this will be used by **HOPR Chat**. In case you see an `Unable to connect to Bootstrap node` message, use one of our other bootstrap nodes. Furthermore, you can also use your own [Infura](https://infura.io/) credentials instead of the ones provided here, but ensure you use the Kovan provider.

{% tabs %}
{% tab title="ch-t-01" %}
```text
docker run -v $(pwd)/db:/app/db \
-e HOST_IPV4=0.0.0.0:9091 \
-e BOOTSTRAP_SERVERS=/dns4/ch-test-01.hoprnet.io/tcp/9091/p2p/16Uiu2HAmMUwDHzmFJaATzQPUFgzry5oxvSgWF2Vc553HCpekC4q \
-e ETHEREUM_PROVIDER=wss://kovan.infura.io/ws/v3/f7240372c1b442a6885ce9bb825ebc36 \
-p 9091:9091 -it hopr/chat -p switzerland
```
{% endtab %}

{% tab title="ch-t-02" %}
```text
docker run -v $(pwd)/db:/app/db \
-e HOST_IPV4=0.0.0.0:9091 \
-e BOOTSTRAP_SERVERS=/dns4/ch-test-02.hoprnet.io/tcp/9091/p2p/16Uiu2HAmVFVHwJs7EqeRUtY6EZTtv379CiwvJgdsDfmdywbKfgAq \
-e ETHEREUM_PROVIDER=wss://kovan.infura.io/ws/v3/f7240372c1b442a6885ce9bb825ebc36 \
-p 9091:9091 -it hopr/chat -p switzerland
```
{% endtab %}
{% endtabs %}

You will be welcomed by **HOPR Chat**’s introductory screen which provides you with further instructions on how to move forward.

![HOPR Chat will prompt you to request funds online.](../../.gitbook/assets/hopr.gif)

**Please copy your HOPR Chat account address,** as you will need it in further steps. **HOPR Chat** has been started bootstrapped successfully, now you need to fund your **HOPR Chat** account with some KETH, [Kovan’s Network](https://kovan-testnet.github.io/website/) testnet ETH tokens. 

## Step 4 - Funding your HOPR Chat Account

Since **HOPR Chat** uses the [Ethereum](https://ethereum.org/) Payment Channels to transfer **HOPR Tokens** as an economic incentive for **HOPR** users, your **HOPR Chat** account requires ETH and **HOPR Tokens**. At the time of writing, **HOPR Chat** works in the Kovan network, so you need the equivalent currency which is free to request in [Kovan's](https://faucet.kovan.network/) and [HOPR](https://faucet.hoprnet.io/) Faucet websites. To request Kovan ETH tokens you will need to have a [GitHub](https://github.com/) account.

Copy your account from Step 3, and paste it in the following websites. 

* Kovan Network Faucet - [https://faucet.kovan.network/](https://faucet.kovan.network/)
* HOPR Network Faucet - [https://faucet.hoprnet.io/](https://faucet.hoprnet.io/)

**HOPR Chat** will not fully initialise until your account has been funded with some Kovan ETH and HOPR. After the tokens have landed in your account, you are ready to use **HOPR Chat.** Execute the same command shared in Step 3 to see the following screen.

![HOPR Chat will tell you its balance in Kovan ETH and HOPR tokens](../../.gitbook/assets/running_hopr_chat_w_balance.gif)

Your **HOPR Chat** instance is ready to be used!

## Step 5 - Sending a HOPR message

With **HOPR Chat** up and running, you can now send messages to any connected nodes in the network. You can either have a friend send you their address, or you can also start another **HOPR Chat** instance. If you choose to start a second instance, you will need to follow Steps 3 and 4 for this new account. You can also find **HOPR Chat** users in our [Telegram channel](https://t.me/hoprnet).

First, ensure you have enough **HOPR Tokens** to send and receive messages. Run `balance` to see the screen from Step 4.   
  
Now, let's find some nodes to talk to. To do this, run `crawl`, which will show you other users available to chat to.

![The crawl command will show you other connected nodes.](../../.gitbook/assets/running_hopr_chat_and_crawling.gif)

To talk to other users, copy another connected user address and send a message to them with the `send` command. This will look something like: `send 16Uiu2HAmCtWxx3Ky3ZjtWj1whkezdRvMAYKU9f57CRPj2FkPtWsD`

**HOPR Chat** will then prompt you for a message to send.

![Your message will be sent privately through the HOPR network](../../.gitbook/assets/running_hopr_chat_and_sending.gif)

Congratulations! You have communicated with another node using a privacy-preserving decentralised protocol. **HOPR Chat** is just a proof of concept right now, but you can already see the capabilities of the protocol.

## Additional Notes

### Bootstrap Nodes

For **HOPR Chat** to work, you need to provide it with at least one **HOPR Chat** node in bootstrap mode. For more information about these nodes and which ones are available, please see our **bootstrap nodes** page.

{% page-ref page="../bootstrap-nodes.md" %}



