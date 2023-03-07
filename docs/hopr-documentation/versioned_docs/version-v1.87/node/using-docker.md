---
id: using-docker
title: Using Docker
---

import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';

:::info INFO
The instructions below are for Linux and macOS, however, due to the nature of Docker, you may also run this on Windows.
:::

:::caution WARNING
The docker image is in alpha stage. The following instructions may not work for you.
:::

The Docker setup allows you quickly get started with **HOPRd** without having to download any other software requirements in your machine. This allows you to quickly get started using the system, but has some hardware requirements to be aware of.

To use Docker, you will need a device that supports hardware-level virtualisation: VT-x for Intel-based PCs and AMD-V for AMD processors. Most of the Mac and Linux machines support it out of the box, so ensure you have enough memory \(e.g. 2 GB\) and disk space \(e.g. 1 GB\) before starting.

## Installing Docker

Before doing anything else, you need to install **Docker Desktop** on your machine. Docker is natively supported in MacOS/Linux, and will prompt you with any additional requirements, depending on your operating system. Depending on your setup, you might need to follow additional steps to ensure your computer works properly with Docker.

<Tabs>
<TabItem value="linux" label="Linux">

Depending of your distribution, please follow the official guidelines for how to install and run Docker on your workstation.

- [Installing Docker in Ubuntu](https://docs.docker.com/engine/install/ubuntu/)
- [Installing Docker in Fedora](https://docs.docker.com/engine/install/fedora/)
- [Installing Docker in Debian](https://docs.docker.com/engine/install/debian/)
- [Installing Docker in CentOS](https://docs.docker.com/engine/install/centos/)

</TabItem>
<TabItem value="mac" label="macOS">

1. Visit [Docker](https://www.docker.com/get-started) and download Docker Desktop to your computer.
2. Follow the wizard steps to ensure Docker is installed.
3. Ensure the installation was successful by running `docker ps` in your terminal.

</TabItem>
</Tabs>

### Downloading HOPRd image using Docker

:::info NOTE

Before downloading HOPRd image and starting a container, make sure the **Docker** is running.

:::

All our docker images can be found in [our Google Cloud Container Registry](https://console.cloud.google.com/gcr/images/hoprassociation/global/hoprd).
Each image is prefixed with `gcr.io/hoprassociation/hoprd`.
The `ouagadougou` tag represents the latest community release version.

Open your console based on your OS:

- Terminal (Mac OS / Linux OS)

Before starting a container, please create your own **Security Token**. Replace **YOUR_SECURITY_TOKEN** with your own and only then paste the command.

:::danger Requirements

Security token should contain:

- at least 8 symbols
- a lowercase letter
- uppercase letter
- a number
- a special symbol (don't use `%` or whitespace)

This ensures the node cannot be accessed by a malicious user residing in the same network.

:::

```bash
docker run --pull always -ti -v $HOME/.hoprd-db:/app/db -p 9091:9091/tcp -p 9091:9091/udp -p 3000:3000 -p 3001:3001 gcr.io/hoprassociation/hoprd:ouagadougou --admin --password 'open-sesame-iTwnsPNg0hpagP+o6T0KOwiH9RQ0' --init --rest --restHost "0.0.0.0" --restPort 3001 --identity /app/db/.hopr-id-ouagadougou --apiToken 'YOUR_SECURITY_TOKEN' --adminHost "0.0.0.0" --adminPort 3000 --host "0.0.0.0:9091"
```

Also all ports are mapped to your local host, assuming you stick to the default port numbers.

The installation process has been finished! Now you can proceed to [Guide using a hoprd node](guide-using-a-hoprd-node).
