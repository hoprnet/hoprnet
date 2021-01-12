---
description: Get familiar with HOPR on your Windows computer.
---

# Windows Quickstart

This quick start tutorial will show you how to use **HOPR** by installing **HOPR Chat** in your system using Docker on a Windows computer. In this step-by-step guide, we will download Docker, run **HOPR Chat,** and send a message to another user connected to the **HOPR Network**.

## Step 1 - Installing Docker in your machine

Before anything, you need to install Docker Desktop in your machine. Depending in your Windows version, you might need to install [Docker Desktop](https://hub.docker.com/editions/community/docker-ce-desktop-windows/) \(Windows 10 Pro or over\) or [Docker Toolbox](https://docs.docker.com/toolbox/overview/) \(Windows 10 Home\). Please bear in mind that in order to get Docker running, your computer needs to have some minimal hardware requirements described in [here](https://docs.docker.com/toolbox/toolbox_install_windows/#step-1-check-your-version).

### Installing Docker on Microsoft Windows 10 Professional or Enterprise

1. Please go to [Docker ](https://www.docker.com/products/docker-desktop)and download Docker Desktop in your computer.
2. Follow-up the wizard steps to ensure Docker is installed.
3. Ensure the installation was successful by running `docker ps`

### Installing Docker on Microsoft Windows 10 Home

1. Go to [Docker Hub](https://docs.docker.com/toolbox/overview/) to download Docker Toolbox in your computer.
2. Follow-up the wizard steps to ensure Docker is installed.
3. Ensure the installation was successful by running `docker ps`

## Step 2 - Downloading HOPR Chat image from Docker Hub

To use **HOPR Chat,** run `docker pull hopr/chat` from your command line \(“cmd.exe”\) or Powershell terminal. Please bear in mind this process will take some time depending on your internet connection.

![](../.gitbook/assets/dockerinstall.gif)

To ensure your machine has successfully downloaded **HOPR Chat,** run `docker images`.You will be shown the **HOPR Chat** image being installed locally, ready to be run.

![HOPR Chat distributed as a Docker image](../.gitbook/assets/image.png)

## Step 3 - **Running HOPR Chat**

To run **HOPR Chat,** you only need to run the following command. You can replace `switzerland` for anything else, but ensure to always use the same password as this will be used by **HOPR Chat**. Furthermore, you can also use your own [Infura](https://infura.io/) credentials instead of the ones provided here, but ensure you use the Kovan provider.

{% tabs %}
{% tab title="ch-t-01" %}

```text
docker run -v %cd%/db:/app/db ^
-e HOST_IPV4=0.0.0.0:9091 ^
-e BOOTSTRAP_SERVERS=/dns4/ch-test-01.hoprnet.io/tcp/9091/p2p/16Uiu2HAmMUwDHzmFJaATzQPUFgzry5oxvSgWF2Vc553HCpekC4qU ^
-e ETHEREUM_PROVIDER=wss://kovan.infura.io/ws/v3/f7240372c1b442a6885ce9bb825ebc36 ^
-p 9091:9091 -it hopr/chat -p switzerland
```

{% endtab %}

{% tab title="ch-t-02" %}

```
docker run -v %cd%/db:/app/db ^
-e HOST_IPV4=0.0.0.0:9091 ^
-e BOOTSTRAP_SERVERS=/dns4/ch-test-02.hoprnet.io/tcp/9091/p2p/16Uiu2HAmVFVHwJs7EqeRUtY6EZTtv379CiwvJgdsDfmdywbKfgAq ^
-e ETHEREUM_PROVIDER=wss://kovan.infura.io/ws/v3/f7240372c1b442a6885ce9bb825ebc36 ^
-p 9091:9091 -it hopr/chat -p switzerland
```

{% endtab %}
{% endtabs %}

You will be welcomed by the following message.

![HOPR Chat will prompt you to request funds online.](../.gitbook/assets/image%20%285%29.png)

**Please copy your HOPR Chat account address,** as you will need it in further steps. **HOPR Chat** has been started bootstrapped successfully, now you need to fund your **HOPR Chat** account with some KETH, [Kovan’s Network](https://kovan-testnet.github.io/website/) testnet ETH tokens.

## Step 4 - Funding your HOPR Chat Account

Since **HOPR Chat** uses the [Ethereum](https://ethereum.org/) Payment Channels to transfer **HOPR Tokens** as an economic incentive for **HOPR** users, your **HOPR Chat** account requires ETH and **HOPR Tokens**. At the time of writing, **HOPR Chat** works in the Kovan network, so you need the equivalent currency which is free to request in [Kovan's](https://faucet.kovan.network/) and [HOPR](https://faucet.hoprnet.io/) Faucet websites. To request Kovan ETH tokens you will need to have a [GitHub](https://github.com/) account.

Copy your account from Step 3, and paste it in the following websites.

- Kovan Network Faucet - [https://faucet.kovan.network/](https://faucet.kovan.network/)
- HOPR Network Faucet - [https://faucet.hoprnet.io/](https://faucet.hoprnet.io/)

**HOPR Chat** will not fully initialize until your account has been funded with some Kovan ETH and HOPR. After the tokens have landed in your account, you are ready to use **HOPR Chat.** Execute the same command shared in Step 3 to see the following screen.

![HOPR Chat fully working after having its account funded](../.gitbook/assets/image%20%281%29.png)

You verify your balance from your **HOPR Chat** account, execute the command `balance` to see the following screen:

![HOPR Chat will tell you its balance in Kovan ETH and HOPR tokens](../.gitbook/assets/image%20%284%29.png)

Your **HOPR Chat** instance is ready to be used!

## Step 5 - Sending a HOPR message

With **HOPR Chat** up and running, you can now send messages to any connected nodes in the network. You can have another friend send you their address, or you can also start another **HOPR Chat** instance. You will need to follow Steps 3 and 4 in this new account in case you decide to go through, but you can also find **HOPR Chat** users in our [Telegram channel](https://t.me/hoprnet).

First, ensure you have enough **HOPR Tokens** to send and receive messages. Run `balance` to see the previous screen from Step 4. Now, let's ensure there are some nodes to talk to. To do so, run `crawl`, which will show you existing users you can talk to.

![The crawl command will show you other connected nodes.](../.gitbook/assets/image%20%283%29.png)

To talk to other users, copy another connected user address and send a message to them with the `send` command. This will look something like `send 16Uiu2HAmCtWxx3Ky3ZjtWj1whkezdRvMAYKU9f57CRPj2FkPtWsD`, which will trigger **HOPR Chat** to prompt you for a message to send.

![Your message will be sent privately through the HOPR network](../.gitbook/assets/image%20%286%29.png)

Congratulations! You have communicated with another individual using a privacy-preserving decentralised protocol. **HOPR Chat** is right now only a Proof-of-Concept but it can already show you the capabilities the protocol can have.

## Additional Notes

### Troubleshooting volume access

**HOPR Chat** requires write access to a working directory to store important data in your computer. If you didn't started your command line with elevated privileges, you might be prompted to give access to your current working directory.

![Windows prompting access to write to your current directory](../.gitbook/assets/image%20%282%29.png)

### Bootstrap Nodes

For **HOPR Chat** to work, you need to make sure you provide it with at least one **HOPR Chat** node in Bootstrap Mode. For more information about these nodes and which ones are available, please see our **Bootstrap Nodes** page.

{% page-ref page="../hopr-chat/bootstrap-nodes.md" %}
