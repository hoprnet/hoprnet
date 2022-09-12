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

You can use Docker to install a `hoprd` node on your device quickly without worrying too much about the operating system or any additional software. There are however some hardware requirements needed in order to complete the installation.

To use Docker, you will need a device that supports hardware-level virtualisation: VT-x for Intel-based PCs and AMD-V for AMD processors. Most of the Mac and Linux machines support it out of the box, so ensure you have enough memory \(e.g. 2 GB\) and disk space \(e.g. 1 GB\) before starting.

You should also make sure your device has the following minimum requirements in order to run the node:

* Dual Core CPU ~ 2 GHz
* 4 GB RAM
* at least 3 GB Disk Space

At leasst 8 GB RAM and 10 GB Disk Space is ideal but not required. 

## Installing Docker

Before doing anything else, you need to install **Docker Desktop** on your machine. Docker is natively supported in MacOS/Linux, you will be promopted with any additional requirements after installing **Docker Desktop**. Depending on your setup, you might need to follow additional steps to ensure Docker runs smoothly on your machine.

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

Before downloading the HOPRd image, make sure **Docker** is installaed.

:::

All our docker images can be found in [our Google Cloud Container Registry](https://console.cloud.google.com/gcr/images/hoprassociation/global/hoprd).
Each image is prefixed with `gcr.io/hoprassociation/hoprd`.
The `paleochora` tag represents the latest community release version.

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
docker run --pull always -ti -v $HOME/.hoprd-db:/app/db -p 9091:9091 -p 3000:3000 -p 3001:3001 gcr.io/hoprassociation/hoprd:paleochora --admin --password 'open-sesame-iTwnsPNg0hpagP+o6T0KOwiH9RQ0' --init --api --apiHost "0.0.0.0" --apiPort 3001 --identity /app/db/.hopr-id-paleochora --apiToken 'YOUR_SECURITY_TOKEN' --adminHost "0.0.0.0" --adminPort 3000 --host "0.0.0.0:9091"
```

Please make note of the `--apiToken` (Security token), as this will be used to access the `hopr-admin`. It may also be a good idea to note the `--password`, incase you want to decrypt your identity file and retrieve your private key or funds later.

**Note:** Withdrawing funds is possible through the `hopr-admin`, this is just a precaution for safe keeping.

All ports are mapped to your local host, assuming you stick to the default port numbers. You should be able to view the `hopr-admin` interface at: [http://localhost:3000](http://localhost:3000) (replace `localhost` with your server ip address if you are using a VPS).

The installation process is now complete! You can proceed to our [hopr-admin tutorial](using-hopr-admin).
