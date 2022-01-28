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

Before doing anything else, you need to install [Docker Desktop](https://hub.docker.com/editions/community/docker-ce-desktop-mac/) on your machine. Docker is natively supported in MacOS/Linux, and will prompt you with any additional requirements, depending on your operating system. Depending on your setup, you might need to follow additional steps to ensure your computer works properly with Docker.

<Tabs>
<TabItem value="linux" label="Linux">

Depending of your distribution, please follow the official guidelines for how to install and run Docker on your workstation.

- [Installing Docker in Ubuntu](https://docs.docker.com/engine/install/ubuntu/)
- [Installing Docker in Fedora](https://docs.docker.com/engine/install/fedora/)
- [Installing Docker in Debian](https://docs.docker.com/engine/install/debian/)
- [Installing Docker in CentOS](https://docs.docker.com/engine/install/centos/)

</TabItem>
<TabItem value="mac" label="macOS">

1. Visit [Docker Hub](https://hub.docker.com/editions/community/docker-ce-desktop-mac) and download Docker Desktop to your computer.
2. Follow the wizard steps to ensure Docker is installed.
3. Ensure the installation was successful by running `docker ps` in your terminal.

</TabItem>
</Tabs>

### Downloading HOPRd image using Docker

All our docker images can be found in [our Google Cloud Container Registry](https://console.cloud.google.com/gcr/images/hoprassociation/global/hoprd).
Each image is prefixed with `gcr.io/hoprassociation/hoprd`.
The `wildhorn-v2` tag represents the latest community release version.

You can pull the Docker image like so:

```bash
docker pull gcr.io/hoprassociation/hoprd:athens
```

Then start a container:

```bash
docker run --pull always -ti -v $HOME/.hoprd-db-athens:/app/db -p 9091:9091 -p 3100:3100 -p 3101:3101 gcr.io/hoprassociation/hoprd:athens --admin --password='open-sesame-iTwnsPNg0hpagP+o6T0KOwiH9RQ0' --init --rest --restHost "0.0.0.0" --restPort 3101 --identity /app/db/.hopr-id-athens --apiToken='<YOUR_SECRET_TOKEN>' --adminHost "0.0.0.0" --adminPort 3100 --host "0.0.0.0:9191"
```

Also all ports are mapped to your local host, assuming you stick to the default port numbers.

:::danger Important

If you want to secure your hoprd admin UI, in the command line you must use **--apiToken** tag.

**<YOUR_SECRET_TOKEN\>** - Replace it with your own password (don't use "<\>").

Password should contain:

- at least 8 symbols
- a lowercase letter
- uppercase letter
- a number
- a special symbol

This ensures the node cannot be accessed by a malicious user residing in the same network.

:::
