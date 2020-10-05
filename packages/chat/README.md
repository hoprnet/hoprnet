<p align="center"><a href="https://hoprnet.org" target="_blank" rel="noopener noreferrer"><img width="100" src="https://github.com/hoprnet/hopr-assets/blob/master/v1/logo/hopr_logo_padded.png?raw=true" alt="HOPR Logo"></a></p>
<p align="center">
  <a href="https://github.com/hoprnet/hopr-core/actions?query=workflow%3A%22Node.js+CI%22"><img src="https://github.com/hoprnet/hopr-chat/workflows/Node.js%20CI/badge.svg" alt="Build Status"></a>
</p>
<h2 align="center">HOPR Chat</h2>

HOPR Chat is a proof of concept and demo application, showing of the
capabilities of the [HOPR](https://github.com/hoprnet/hopr-core) protocol, allow you to start a **HOPR Node** and connect to the **HOPR Network**.

## Installing

### From releases

Go to [Releases](https://github.com/hoprnet/hopr-chat/releases) to install the application from a pre-compiled binary. For further instructions on how to do so, please read our [quick start](https://docs.hoprnet.org/home/getting-started/hopr-chat/quickstart) guide or for more experienced users try the [advanced setup](https://docs.hoprnet.org/home/getting-started/hopr-chat/setup).

### From docker images

- Stable - [Docker Hub](https://hub.docker.com/r/hopr/chat)
- Alpha - [Google Registry](https://gcr.io/hoprassociation/hopr-chat)

Pick an image from any Docker Registry described above, and pull it (e.g. `docker pull hopr/chat`). To run it with the proper bindings, you can do the following:

```
docker run -v $(pwd)/db:/app/db \
-e HOST_IPV4=0.0.0.0:9091 \
-e BOOTSTRAP_SERVERS=/ip4/34.65.219.148/tcp/9091/p2p/16Uiu2HAkwSEiK819yvnG84pNFsqXkpFX4uiCaNSwADnmYeAfctRn,/ip4/34.65.148.229/tcp/9091/p2p/16Uiu2HAmRsp3VBLcyPfTBkJYEwS47bewxWqqm4sEpJEtPBLeV93n \
-e ETHEREUM_PROVIDER=wss://kovan.infura.io/ws/v3/f7240372c1b442a6885ce9bb825ebc36 \
-p 9091:9091 -it hopr/chat -p switzerland
```

For further informations on how to run **HOPR Chat** using Docker, please see our [advanced setup](https://docs.hoprnet.org/home/getting-started/hopr-chat/setup).

### From source code

To install **HOPR Chat** from source code, clone this repository:

```
git clone git@github.com:hoprnet/hopr-chat.git
```

## Development

To contribute to the project, please follow the next steps to learn how to install the source code's dependencies, compile the project from scratch, and run it locally from your computer.

### Install depedencies

```
yarn
```

### Compiling source code

```
yarn build
```

### Run application

```
HOST_IPV4=0.0.0.0:9091
ETHEREUM_PROVIDER=wss://kovan.infura.io/ws/v3/f7240372c1b442a6885ce9bb825ebc36
yarn start
```

When the service is running you can type `help` to see a full list of commands.
