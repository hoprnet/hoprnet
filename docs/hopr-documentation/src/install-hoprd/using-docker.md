```eval_rst
.. ATTENTION::
   The instructions below are for Linux and macOS, however, due to the nature of Docker, you may also run this on Windows.
```

```eval_rst
.. WARNING::
   The docker image is in alpha stage. The following instructions may not work for you.
```

# Using Docker

The Docker setup allows you quickly get started with **HOPRd** without having to download any other software requirements in your machine. This allows you to quickly get started using the system, but has some hardware requirements to be aware of.

To use Docker, you will need a device that supports hardware-level virtualisation: VT-x for Intel-based PCs and AMD-V for AMD processors. Most of the Mac and Linux machines support it out of the box, so ensure you have enough memory \(e.g. 2 GB\) and disk space \(e.g. 1 GB\) before starting.

## Installing Docker

Before doing anything else, you need to install [Docker Desktop](https://hub.docker.com/editions/community/docker-ce-desktop-mac/) on your machine. Docker is natively supported in MacOS/Linux, and will prompt you with any additional requirements, depending on your operating system. Depending on your setup, you might need to follow additional steps to ensure your computer works properly with Docker.

```eval_rst
.. content-tabs::

    .. tab-container:: linux
        :title: Linux

        Depending of your distribution, please follow the official guidelines for how to install and run Docker on your workstation.

        - `Installing Docker in Ubuntu <https://docs.docker.com/engine/install/ubuntu/>`_
        - `Installing Docker in Fedora <https://docs.docker.com/engine/install/fedora/>`_
        - `Installing Docker in Debian <https://docs.docker.com/engine/install/debian/>`_
        - `Installing Docker in CentOS <https://docs.docker.com/engine/install/centos/>`_

    .. tab-container:: macos
        :title: macOS

        1. Visit `Docker Hub <https://hub.docker.com/editions/community/docker-ce-desktop-mac/>`_ and download Docker Desktop to your computer.
        2. Follow the wizard steps to ensure Docker is installed.
        3. Ensure the installation was successful by running `docker ps` in your terminal.
```

## Downloading HOPRd image using Docker

To use **HOPRd,** run `docker pull gcr.io/hoprassociation/hoprd` from your terminal. This process may take some time depending on your internet connection.

![Currently HOPRd is about ~1.25 GB, please be patient.](../../images/docker_install_macos.gif)

To ensure your machine has successfully downloaded **HOPRd,** run `docker images`.You will be shown the **HOPRd** image being installed locally, ready to be run.

![HOPR Chat distributed as a Docker image](../../images/docker_images.gif)
