---
id: myne-chat
title: myne.chat (alpha version)
---

myne.chat is the first dApp built on top of the HOPR protocol. This alpha version will let you run a localized version where you can test how myne works in ideal conditions.

This tutorial will walk you through running a local cluster with five instances of myne.chat. You can then share myne.chat links with your friends or colleagues and start chatting.

Note that although this version of myne.chat uses the HOPR protocol, it is not fully private, because:

- the small network provides minimal mixing
- there is no cover traffic
- the person who sets up the cluster and distributes the links can use them to enter any instance and read the conversations.

Please inform your friends or colleagues of this before sharing the links.

## How to start a chat?

Before starting your localized network, make sure you have registered an account on [https://github.com](https://github.com)

Start a localized network with the help of gitpod: [https://gitpod.io/#https://github.com/hoprnet/myne-chat](https://gitpod.io/#https://github.com/hoprnet/myne-chat)

A new window will open with a Gitpod environment. You will need to wait while it installs all the packages and dependencies. **This will take several minutes**.

![first dapp myne chat](/img/dapps/myne-chat-alpha-1.jpg)

(**1**) Status window showing installation progress

![first dapp myne chat](/img/dapps/myne-chat-alpha-2.jpg)

(**2**) Multiple ports will start to appear on this status bar.

(**3**) Notifications (**A service is available on port …**) will pop up in the bottom right corner. Please ignore them all and continue to the next step.

![first dapp myne chat](/img/dapps/myne-chat-alpha-3.jpg)

(**4**) In this middle panel you’ll see nodes being fetched and channels being opened and funded. When you see a list of **node1**, **node2 …** up to **node5** with links to Myne Chat, Rest API, Admin UI, it means the localized network has been deployed completely and you can move to the next step.

![first dapp myne chat](/img/dapps/myne-chat-alpha-4.jpg)

(**5**) Navigate to the bottom right panel and click the entry **(Mynechat) URLs: bash**

(**6**) On the left side panel under the terminal tab, you will see users along with details of their MyneChat, HOPRd API and HOPRd node UI. Please scroll up to see the first user, Alex.

In total you will have 5 users: **Alex**, **Betty**, **Chão**, **Dmytro**, **Zoe**

Let’s take a look at Alex’s details we have:

:::info Reminder

These links will not work for you, because you will be running your own localized network which will generate different links.

:::

**MyneChat**: [https://8080-hoprnet-mynechat-b4ssakdaol7.ws-eu39b.gitpod.io/?httpEndpoint=https://13301-hoprnet-mynechat-b4ssakdaol7.ws-eu39b.gitpod.io&wsEndpoint=wss://19501-hoprnet-mynechat-b4ssakdaol7.ws-eu39b.gitpod.io&securityToken=^^LOCAL-testing-123^^](https://8080-hoprnet-mynechat-b4ssakdaol7.ws-eu39b.gitpod.io/?httpEndpoint=https://13301-hoprnet-mynechat-b4ssakdaol7.ws-eu39b.gitpod.io&wsEndpoint=wss://19501-hoprnet-mynechat-b4ssakdaol7.ws-eu39b.gitpod.io&securityToken=^^LOCAL-testing-123^^)

**Rest API**: [https://13301-hoprnet-mynechat-b4ssakdaol7.ws-eu39b.gitpod.io/api/v2/\_swagger](https://13301-hoprnet-mynechat-b4ssakdaol7.ws-eu39b.gitpod.io/api/v2/_swagger)

**Admin UI**: [https://19501-hoprnet-mynechat-b4ssakdaol7.ws-eu39b.gitpod.io/](https://19501-hoprnet-mynechat-b4ssakdaol7.ws-eu39b.gitpod.io/)

![first dapp myne chat](/img/dapps/myne-chat-alpha-5.jpg)

(**7**) To run Alex’s myne.chat instance, just copy the MyneChat link and paste into a new browser tab.

![first dapp myne chat](/img/dapps/myne-chat-alpha-6.jpg)

Congrats! You have launched the Alex MyneChat app locally! Now you can share the MyneChat links for **Betty**, **Chão**, **Dmytro**, **Zoe** with your friends or colleagues and start a conversation with them.

## How does chat work?

Every MyneChat user has their own **User Peer ID**. This is how you connect with each other.
To see Alex’s peer ID, navigate to the top left corner and select the bars icon.

![first dapp myne chat](/img/dapps/myne-chat-alpha-7.jpg)

(**1**) In this case, Alex’s Peer ID is: **16Uiu2HAmNT7p3t1HpRKPehHNeb4RvsKMFMnyiD4ekA9utKHj6UQC**

To start a conversation you will need to know your friends’ peer ID. For this alpha version, you will need to ask for it outside of myne.chat. Once you have it, navigate to the top left corner and click the plus sign to enter your friend’s peer ID. You can now start to send them messages.

Are you a developer? Learn how to build on top of the HOPR protocol with our [HOPR Cluster Development Setup Guide](/developers/starting-local-cluster).
