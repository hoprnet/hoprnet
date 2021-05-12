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
  - [Using NPM](#using-npm)
  - [Using Docker](#using-docker)
  - [Using Nix package manager](#using-nix-package-manager)
- [Usage](#usage)
  - [Starting database](#starting-database)
  - [Starting node with custom port](#starting-node-with-custom-port)
- [Develop](#develop)
- [Tooling](#tooling)
- [Contact](#contact)
- [License](#license)

## Getting Started

A good place to start is the
[Getting Started guide on YouTube][7] which walks through the following
instructions using GitPod.

## Install

### Using NPM

Using the [hoprd npm package][6]:

```sh
npm install -g @hoprnet/hoprd
hoprd --admin
```

### Using Docker

All our docker images can be found in [our Google Cloud Container Registry][4].
Each image is prefixed with `gcr.io/hoprassociation/$PROJECT:$RELEASE`.
The `latest` tag represents the `master` branch.
Stable releases are published on [Docker Hub][5].

For ease of use you can set up a shell alias to run the latest docker container:

```sh
alias hoprd='docker run --pull always --rm -v $(pwd)/db:/app/db gcr.io/hoprassociation/hoprd:latest'
```

### Using [Nix package manager][1]

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

## Usage

### Starting database

```sh
hoprd --admin --init
```

### Starting node with custom port

```sh
hoprd --admin --host="0.0.0.0:1291"
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
[5]: https://hub.docker.com/u/hopr
[6]: https://www.npmjs.com/package/@hoprnet/hoprd
[7]: https://www.youtube.com/watch?v=d0Eb6haIUu4
