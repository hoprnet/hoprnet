<p align="center"><a href="https://hoprnet.org" target="_blank" rel="noopener noreferrer"><img width="100" src="https://github.com/hoprnet/hopr-assets/blob/master/v1/logo/hopr_logo_padded.png?raw=true" alt="HOPR Logo"></a></p>
<p align="center">
  <a href="https://github.com/hoprnet/hopr-core/actions?query=workflow%3A%22Node.js+CI%22"><img src="https://github.com/hoprnet/hopr-chat/workflows/Node.js%20CI/badge.svg" alt="Build Status"></a>
</p>
<h2 align="center">HOPR Chat</h2>

HOPR Chat is a proof of concept and demo application, showing of the
capabilities of the [HOPR](https://github.com/hoprnet/hopr-core) protocol, allow you to start a **HOPR Node** and connect to the **HOPR Network**.

## Docker Images

- Stable - [Docker Hub](https://hub.docker.com/r/hopr/chat)
- Alpha - [Google Registry](https://gcr.io/hoprassociation/hopr-chat)

## Installing
```
yarn
```

### Compiling
```
yarn build
```

## Running

```
HOST_IPV4=0.0.0.0:9091
ETHEREUM_PROVIDER=wss://kovan.infura.io/ws/v3/f7240372c1b442a6885ce9bb825ebc36
yarn start
```

When the service is running you can type `help` to see a full list of commands.
