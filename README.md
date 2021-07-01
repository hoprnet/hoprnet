<!-- INTRODUCTION -->
<p align="center">
  <a href="https://hoprnet.org" target="_blank" rel="noopener noreferrer">
    <img width="100" src="https://github.com/hoprnet/hopr-assets/blob/master/v1/logo/hopr_logo_padded.png?raw=true" alt="HOPR Logo">
  </a>
  
  <!-- Title Placeholder -->
  <h3 align="center">HOPR</h3>
  <p align="center">
    <code>A project by the HOPR Association</code>
  </p>
  <p align="center">
    HOPR is a privacy-preserving messaging protocol which enables the creation of a secure communication network via relay nodes powered by economic incentives using digital tokens.
  </p>
  <p align="center">
    <img src="https://img.shields.io/badge/Gitpod-ready--to--code-blue?logo=gitpod" alt="Gitpod">
  </p>
</p>

## Table of Contents

- [Table of Contents](#table-of-contents)
- [Getting Started](#getting-started)
- [Install](#install)
  - [Install via NPM](#install-via-npm)
  - [Install via Docker](#install-via-docker)
  - [Install via Nix package manager](#install-via-nix-package-manager)
- [Using](#using)
  - [Using NPM](#using-npm)
  - [Using Docker](#using-docker)
- [Develop](#develop)
- [Test](#test)
  - [Github Actions CI](#github-actions-ci)
  - [End-to-End Test](#end-to-end-test)
- [Tooling](#tooling)
- [Contact](#contact)
- [License](#license)

## Getting Started

A good place to start is the
[Getting Started guide on YouTube][7] which walks through the following
instructions using GitPod.

## Install

The following instructions show how the latest community release may be
installed. The instructions should be adapted if you want to use the latest
development release or any other older release.

### Install via NPM

Using the [hoprd npm package][6]:

```sh
mkdir MY_NEW_HOPR_TEST_FOLDER
cd MY_NEW_HOPR_TEST_FOLDER
npm install @hoprnet/hoprd@1.72
```

### Install via Docker

All our docker images can be found in [our Google Cloud Container Registry][4].
Each image is prefixed with `gcr.io/hoprassociation/$PROJECT:$RELEASE`.
The `latest` tag represents the `master` branch, while the `latest-paphos` tag
represents the most recent `release/*` branch.

You can pull the Docker image like so:

```sh
docker pull gcr.io/hoprassociation/hoprd:latest-paphos
```

For ease of use you can set up a shell alias to run the latest release as a docker container:

```sh
alias hoprd='docker run --pull always -ti -v ${HOPRD_DATA_DIR:-$HOME/.hoprd-db}:/app/db -p 9091:9091 -p 3000:3000 -p 3001:3001 gcr.io/hoprassociation/hoprd:latest-paphos'
```

**IMPORTANT:** Using the above command will map the database folder used by hoprd to a local folder called `.hoprd-db` in your home directory. You can customize the location of that folder further by executing the following command:

```sh
HOPRD_DATA_DIR=${HOME}/.hoprd-better-db-folder eval hoprd
```

Also all ports are mapped to your local host, assuming you stick to the default port numbers.

### Install via [Nix package manager][1]

NOTE: This setup should only be used for development or if you know what you
are doing and don't neetd further supported. Otherwise you should use the `npm`
or `docker` setup.

You will need to clone the `hoprnet` repo first:

```sh
git clone https://github.com/hoprnet/hoprnet
```

If you have [direnv][2] and [lorri][3] set up properly your `nix-shell` will be
configured automatically upon entering the `hoprnet` directory and enabling it
via `direnv allow`. Otherwise you must enter the `nix-shell` manually:

```sh
nix-shell
```

Now you may follow the instructions in [Develop](#develop).

## Using

The `hoprd` provides various command-line switches to configure its behaviour. For reference these are documented here as well:

```sh
$ hoprd --help
Options:
  --help                        Show help  [boolean]
  --version                     Show version number  [boolean]
  --network                     Which network to run the HOPR node on  [choices: "ETHEREUM"] [default: "ETHEREUM"]
  --provider                    A provider url for the Network you specified  [default: "wss://still-patient-forest.xdai.quiknode.pro/f0cdbd6455c0b3aea8512fc9e7d161c1c0abf66a/"]
  --host                        The network host to run the HOPR node on.  [default: "0.0.0.0:9091"]
  --announce                    Announce public IP to the network  [boolean] [default: false]
  --admin                       Run an admin interface on localhost:3000  [boolean] [default: false]
  --rest                        Run a rest interface on localhost:3001  [boolean] [default: false]
  --restHost                    Updates the host for the rest server  [default: "localhost"]
  --restPort                    Updates the port for the rest server  [default: 3001]
  --healthCheck                 Run a health check end point on localhost:8080  [boolean] [default: false]
  --healthCheckHost             Updates the host for the healthcheck server  [default: "localhost"]
  --healthCheckPort             Updates the port for the healthcheck server  [default: 8080]
  --password                    A password to encrypt your keys  [default: ""]
  --identity                    The path to the identity file  [default: "/root/.hopr-identity"]
  --run                         Run a single hopr command, same syntax as in hopr-admin  [default: ""]
  --dryRun                      List all the options used to run the HOPR node, but quit instead of starting  [boolean] [default: false]
  --data                        manually specify the database directory to use  [default: ""]
  --init                        initialize a database if it doesn't already exist  [boolean] [default: false]
  --adminHost                   Host to listen to for admin console  [default: "localhost"]
  --adminPort                   Port to listen to for admin console  [default: 3000]
  --testAnnounceLocalAddresses  For testing local testnets. Announce local addresses.  [boolean] [default: false]
  --testPreferLocalAddresses    For testing local testnets. Prefer local peers to remote.  [boolean] [default: false]
```

As you might have noticed running the node without any command-line argument might not work depending on the installation method used. Here are examples to run a node with some safe configurations set.

### Using NPM

The following command assumes you've setup a local installation like described in [Install via NPM](#install-via-npm).

```sh
cd MY_NEW_HOPR_TEST_FOLDER
DEBUG=hopr* npx hoprd --admin --init --announce
```

Here is a short break-down of each argument.

```sh
hoprd
  --admin   	                         # enable the node's admin UI, available at localhost:3000
  --init 				 # initialize the database and identity if not present
  --announce 				 # announce the node to other nodes in the network and act as relay if publicly reachable
```

### Docker

The following command assumes you've setup an alias like described in [Install via Docker](#install-via-docker).

```sh
hoprd --identity /app/db/.hopr-identity --password switzerland --init --announce --host "0.0.0.0:9091" --admin --adminHost 0.0.0.0
```

Here is a short break-down of each argument.

```sh
hoprd
  --identity /app/db/.hopr-identity      # store your node identity information in the persisted database folder
  --password switzerland   		 # set the encryption password for your identity
  --init 				 # initialize the database and identity if not present
  --announce 				 # announce the node to other nodes in the network and act as relay if publicly reachable
  --host "0.0.0.0:9091"   		 # set IP and port of the P2P API to the container's external IP so it can be reached on your host
  --admin   	                         # enable the node's admin UI
  --adminHost 0.0.0.0                    # set IP of the Rest API to the container's external IP so it can be reached on your host
```

## Develop

```sh
yarn          # Install lerna and sets project up
yarn build    # Builds contracts, clients, etc

# starting network
yarn run:network

# running normal node alice (separate terminal)
DEBUG=hopr* yarn run:hoprd:alice

# running normal node bob (separate terminal)
DEBUG=hopr* yarn run:hoprd:bob

# fund all your nodes to get started
yarn run:faucet:all
```

## Test

### Github Actions CI

We run a fair amount of automation using Github Actions. To ease development
of these workflows one can use [act][8] to run workflows locally in a
Docker environment.

E.g. running the build workflow:

```sh
act -j build
```

For more information please refer to [act][8]'s documentation.

### End-to-End Test

End-to-end testing is usually performed by the CI, but can also be performed
locally by executing:

```sh
./scripts/run-integration-tests-locally.sh
```

## Tooling

As some tools are only partially supported, please tag the respective team member
whenever you need an issue about a particular tool.

| Maintainer       | Technology  |
| :--------------- | :---------: |
| @jjperezaguinaga | Visual Code |
| @tolbrino        |     Nix     |

## Contact

- [Twitter](https://twitter.com/hoprnet)
- [Telegram](https://t.me/hoprnet)
- [Medium](https://medium.com/hoprnet)
- [Reddit](https://www.reddit.com/r/HOPR/)
- [Email](mailto:contact@hoprnet.org)
- [Discord](https://discord.gg/5FWSfq7)
- [Youtube](https://www.youtube.com/channel/UC2DzUtC90LXdW7TfT3igasA)

## License

[GPL v3](LICENSE) © HOPR Association

[1]: https://nixos.org/learn.html
[2]: https://search.nixos.org/packages?channel=20.09&show=direnv&from=0&size=50&sort=relevance&query=direnv
[3]: https://search.nixos.org/packages?channel=20.09&show=lorri&from=0&size=50&sort=relevance&query=lorri
[4]: https://console.cloud.google.com/gcr/images/hoprassociation/GLOBAL
[6]: https://www.npmjs.com/package/@hoprnet/hoprd
[7]: https://www.youtube.com/watch?v=d0Eb6haIUu4
[8]: https://github.com/nektos/act
