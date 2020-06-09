---
description: Get familiar with HOPR on your Mac.
---

# MacOS Quickstart

This quick start tutorial will show you how to use **HOPR** by installing **HOPR Chat** in your system using Docker on a MacOS computer. In this step-by-step guide, we will download Docker, run **HOPR Chat,** and send a message to another user connected to the **HOPR Network**.

## Step 1 - Installing Docker in your machine

Before anything, you need to install [Docker Desktop](https://hub.docker.com/editions/community/docker-ce-desktop-mac/) on your machine. Docker is natively supported in MacOS, and will prompt any install requirements you might need.

### Instructions for installing Docker in MacOS

1. Please go to [Docker Hub ](https://hub.docker.com/editions/community/docker-ce-desktop-mac/)and download Docker Desktop in your computer.
2. Follow-up the wizard steps to ensure Docker is installed.
3. Ensure the installation was successful by running `docker ps` in your terminal.

## Step 2 - Downloading HOPR Chat image from Docker Hub

To use **HOPR Chat,** run `docker pull hopr/chat` from your Terminal. Please bear in mind this process will take some time depending on your internet connection.

![Currently HOPR Chat is about ~0.5 GB, please be patient.](../../.gitbook/assets/docker_install_macos.gif)

To ensure your machine has successfully downloaded **HOPR Chat,** run `docker images`.You will be shown the **HOPR Chat** image being installed locally, ready to be run.

![HOPR Chat distributed as a Docker image](../../.gitbook/assets/docker_images.gif)

## Step 3 - **Running HOPR Chat**

To run **HOPR Chat,** you only need to copy and paste the following command. You can replace `switzerland` for anything else, but ensure to always use the same password as this will be used by **HOPR Chat**. Furthermore, you can also use your own [Infura](https://infura.io/) credentials instead of the ones provided here, but ensure you use the Kovan provider.

```text
docker run -v $(pwd)/db:/app/db \
-e HOST_IPV4=0.0.0.0:9091 \
-e BOOTSTRAP_SERVERS=/dns4/ch-test-01.hoprnet.io/tcp/9091/p2p/16Uiu2HAmThyWP5YWutPmYk9yUZ48ryWyZ7Cf6pMTQduvHUS9sGE7 \
-e ETHEREUM_PROVIDER=wss://kovan.infura.io/ws/v3/f7240372c1b442a6885ce9bb825ebc36 \
-p 9091:9091 -it hopr/chat -p switzerland
```

You will be welcomed by **HOPR Chat**’s introductory screen which provides you with further instructions on how to move forward.

![HOPR Chat will prompt you to request funds online.](../../.gitbook/assets/hopr.gif)

**Please copy your HOPR Chat account address,** as you will need it in further steps. **HOPR Chat** has been started bootstrapped successfully, now you need to fund your **HOPR Chat** account with some KETH, [Kovan’s Network](https://kovan-testnet.github.io/website/) testnet ETH tokens. 

## Step 4 - Funding your HOPR Chat Account

Since **HOPR Chat** uses the [Ethereum](https://ethereum.org/) Payment Channels to transfer **HOPR Tokens** as an economic incentive for **HOPR** users, your **HOPR Chat** account requires ETH and **HOPR Tokens**. At the time of writing, **HOPR Chat** works in the Kovan network, so you need the equivalent currency which is free to request in [Kovan's](https://faucet.kovan.network/) and [HOPR](https://faucet.hoprnet.io/) Faucet websites. To request Kovan ETH tokens you will need to have a [GitHub](https://github.com/) account.

Copy your account from Step 3, and paste it in the following websites. 

* Kovan Network Faucet - [https://faucet.kovan.network/](https://faucet.kovan.network/)
* HOPR Network Faucet - [https://faucet.hoprnet.io/](https://faucet.hoprnet.io/)

**HOPR Chat** will not fully initialise until your account has been funded with some Kovan ETH and HOPR. After the tokens have landed in your account, you are ready to use **HOPR Chat.** Execute the same command shared in Step 3 to see the following screen.



