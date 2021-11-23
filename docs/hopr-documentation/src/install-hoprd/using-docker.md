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

### Downloading HOPRd image using Docker

All our docker images can be found in [our Google Cloud Container Registry][4].
Each image is prefixed with `gcr.io/hoprassociation/$PROJECT:$RELEASE`.
The `master-goerli` tag represents the `master` branch, while the `tuttlingen` tag
represents the most recent `release/*` branch.

You can pull the Docker image like so:

```sh
docker pull gcr.io/hoprassociation/hoprd:tuttlingen
```

For ease of use you can set up a shell alias to run the latest release as a docker container:

```sh
alias hoprd='docker run --pull always -ti -v ${HOPRD_DATA_DIR:-$HOME/.hoprd-db}:/app/db -p 9091:9091 -p 3000:3000 -p 3001:3001 gcr.io/hoprassociation/hoprd:tuttlingen'
```

**IMPORTANT:** Using the above command will map the database folder used by hoprd to a local folder called `.hoprd-db` in your home directory. You can customize the location of that folder further by executing the following command:

```sh
HOPRD_DATA_DIR=${HOME}/.hoprd-better-db-folder eval hoprd
```

Also all ports are mapped to your local host, assuming you stick to the default port numbers.
