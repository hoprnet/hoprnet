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

## Instructions

### Installing a binary from NPM

```sh
npm install -g @hoprnet/hoprd
hoprd --admin
```

### Running in a docker container

We maintain a docker container at
`gcr.io/hoprassociation/hoprd:$RELEASE`, `latest` tags `master`. To use our Docker images, first pull the image and then proceed to alias it as follows:

```sh
docker pull gcr.io/hoprassociation/hoprd
alias hoprd='docker run --rm -v $(pwd)/db:/app/db gcr.io/hoprassociation/hoprd:latest'
```

### Flags

#### Starting database

```sh
hoprd --admin --init
```

#### Running with a custom host (HOPR Node)

```sh
hoprd --admin --host="0.0.0.0:1291"
```

### Developing

```sh
yarn          # Install lerna and sets project up
yarn build    # Builds contracts, clients, etc

# starting network
yarn run:network

# running bootstrap node (separate terminal)
DEBUG=hopr* yarn run:hoprd:bootstrap

# running normal node alice (separate terminal)
DEBUG=hopr* yarn run:hoprd:alice

# running normal node bob (separate terminal)
DEBUG=hopr* yarn run:hoprd:bob

# fund all your nodes to get started
yarn run:faucet:all
```

### Docker images

All our docker images can be found [here](https://console.cloud.google.com/gcr/images/hoprassociation/GLOBAL) and are prefixed as `gcr.io/hoprassociation/$PROJECT:$RELEASE`. Stable releases live in [Docker Hub](https://hub.docker.com/u/hopr)

<!-- CONTACT -->

## Contact

- Twitter - https://twitter.com/hoprnet
- Telegram - https://t.me/hoprnet
- Medium - https://medium.com/hoprnet
- Reddit - https://www.reddit.com/r/HOPR/
- Email - contact@hoprnet.org
- Discord - https://discord.gg/5FWSfq7
- Youtube - https://www.youtube.com/channel/UC2DzUtC90LXdW7TfT3igasA
