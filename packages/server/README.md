<!-- INTRODUCTION -->
<p align="center">
  <a href="https://hoprnet.org" target="_blank" rel="noopener noreferrer">
    <img width="100" src="https://github.com/hoprnet/hopr-assets/blob/master/v1/logo/hopr_logo_padded.png?raw=true" alt="HOPR Logo">
  </a>
  
  <!-- Title Placeholder -->
  <h3 align="center">HOPR Server</h3>
  <p align="center">
	<code>Nest.js-based gRPC-enabled server</code>
  </p>
  <p align="center">
	  HOPR Server is a TypeScript-coded, gRPC server that allows developers to interact with a HOPR Node via a protobuf enabled API. Powered by the <b>HOPR Network</b> and the <a href="https://hoprnet.org" target="_blank" rel="noopener noreferrer">HOPR Association</a>
  </p>
</p>

<!-- BADGES -->
<p align="center">
  <a href="#"><img src="https://github.com/hoprnet/hopr-server/workflows/Node.js%20CI/badge.svg" alt="Build"></a>
</p>

<!-- TABLE OF CONTENTS -->

## Table of Contents

- [Table of Contents](#table-of-contents)
- [Overview](#overview)
  - [Technologies](#technologies)
- [Getting Started](#getting-started)
  - [Prerequisites](#prerequisites)
  - [Installation](#installation)
  - [Docker Image](#docker-image)
    - [Pull latest version](#pull-latest-version)
    - [Run latest version](#run-latest-version)
    - [Run latest version w/specific `BOOTSTRAP_SERVERS`](#run-latest-version-wspecific-bootstrapservers)
- [Usage](#usage)
  - [Commands](#commands)
  - [Environment Variables](#environment-variables)
- [Roadmap](#roadmap)
- [Additional Information](#additional-information)
  - [Query the server using BloomRPC (Recommended)](#query-the-server-using-bloomrpc-recommended)
    - [Quering](#quering)
    - [Sending & Listening to messages](#sending--listening-to-messages)
  - [Querying the server using gCURL](#querying-the-server-using-gcurl)
  - [Considerations](#considerations)
  - [Links](#links)
- [Contributors](#contributors)
- [Contact](#contact)

<!-- OVERVIEW -->

## Overview

**HOPR Server** functions as a wrapper for a **HOPR Node**. Upon start, it will automatically spin a **HOPR Node** listening to port `9091` and create a gRPC-ready interface listening to port `50051` which automatically connects to the **HOPR Network** using the `BOOTSTRAP_SERVERS` defined.

To interact with **HOPR Server**, you need to use [**HOPR Protos**](https://github.com/hoprnet/hopr-protos), the _Protobuf_ implementation of the **HOPR Protocol** which exposes its API via gRPC. **HOPR Protos** can be installed and used via `nom` by installing the [distribution package](https://www.npmjs.com/package/@hoprnet/hopr-protos).

### Technologies

- [HOPR Core](https://github.com/hoprnet/hopr-core)
- [typescript](https://www.typescriptlang.org/)
- [nest.js](https://nestjs.com/)
- [protocol buffers](https://developers.google.com/protocol-buffers)

<!-- GETTING STARTED -->

## Getting Started

To get a local copy up and running follow these simple steps. In case you just want to have a runnable version of **HOPR Server**, you can use our [Docker Image](#docker-image) to quickly start the server.

### Prerequisites

This is an example of how to list things you need to use the software and how to install them.

- [node.js v>=12](https://nodejs.org/)
- [yarn](https://yarnpkg.com/)
- [docker](https://www.docker.com/) (optional)

### Installation

1. Clone the repo

```sh
git clone https://github.com/hoprnet/hopr-server.git
```

2. Install NPM packages

```sh
yarn
```

### Docker Image

#### Pull latest version

```sh
docker pull gcr.io/hoprassociation/hopr-server
```

#### Run latest version

```sh
docker run -p 50051:50051 -p 9091:9091 -it gcr.io/hoprassociation/hopr-server
```

#### Run latest version w/specific `BOOTSTRAP_SERVERS`

```sh
docker run \
  -p 50051:50051 -p 9091:9091 \
  -e BOOTSTRAP_SERVERS=/ip4/34.65.75.45/tcp/9091/p2p/16Uiu2HAm2cjqsDMmprtN2QKaq3LJrq3YK7vtdbQQFsxGrhLRoYsy,/ip4/34.65.177.154/tcp/9091/p2p/16Uiu2HAm9C4oJPeRkdXnYxtXzFpDqpcCbWLsgNW4irrCLZTJ7cBd \
  -it gcr.io/hoprassociation/hopr-server
```

<!-- USAGE EXAMPLES -->

## Usage

Upon installing, you can run `yarn start` to start the server which will use `ts-node`. Once you see `:: HOPR Core Node Started ::`, it means that the server has successfully connected to the **HOPR Network** using the defined `BOOTSTRAP_SERVERS` and is ready to accept requests.

In case you are looking to distribute the application, you can precompile it using `yarn build`, which will compile the `nest.js` TypeScript files and create a `dist` folder, which can then be used by `node` directly.

### Commands

1. `yarn start` - Starts **HOPR Server** in `dev` mode.
2. `yarn build` - Builds **HOPR Server** for production.

_For more information about the HOPR Network, please refer to the [Documentation](https://docs.hoprnet.org)_

### Environment Variables

The following environment variables can be stored and used in an `.env` file located at the root of the project.

| Name              | Description                                                  | Type             | Example                                                      |
| :---------------- | :----------------------------------------------------------- | :--------------- | :----------------------------------------------------------- |
| SERVER_HOST       | server HOST url                                              | string           | 0.0.0.0:50051                                                |
| DEBUG_MODE        | passed to hopr-core: run in debug mode                       | boolean          | TRUE                                                         |
| ID                | passed to hopr-core: demo account ID                         | integer          | 1                                                            |
| BOOTSTRAP_NODE    | passed to hopr-core: TRUE if node is a boostrap node         | boolean          | FALSE                                                        |
| CORE_HOST         | passed to hopr-core: hopr-core HOST url                      | string           | 0.0.0.0:9091                                                 |
| BOOTSTRAP_SERVERS | passed to hopr-core: a list of bootstap server to connect to | array of strings | [src](./src/core/core.service.ts)                            |
| PROVIDER          | passed to hopr-core: blockchain endpoint                     | string           | wss://kovan.infura.io/ws/v3/f7240372c1b442a6885ce9bb825ebc36 |

<!-- ROADMAP -->

## Roadmap

See the [issues](https://github.com/hoprnet/hopr-server/issues) for a list of proposed features (and known issues).

<!-- ADDITIONAL INFORMATION -->

## Additional Information

### Query the server using [BloomRPC](https://github.com/uw-labs/bloomrpc) (Recommended)

1. Download and install [BloomRPC](https://github.com/uw-labs/bloomrpc/releases).
2. Download the `.proto` [files](https://github.com/hoprnet/hopr-protos) that are used by our server.
3. Import `.proto` files by clicking on the top-left `+` icon and navigating to previously downloaded files.
4. Set the server url in the input at top-center to `127.0.0.1:50051`

#### Quering

1. Select on of the unary methods from the left panel, for example: `version.proto -> version.Version -> GetVersion`
2. Click "play" (▶️), you should get a response of something like:

```json
{
  "components_version": {
    "0": "@hoprnet/hopr-core,0.6.21",
    "1": "@hoprnet/hopr-core-connector-interface,1.3.2-35127fb",
    "2": "@hoprnet/hopr-core-ethereum,0.0.12-refactor.473415f",
    "3": "@hoprnet/hopr-utils,0.1.7-c598e77",
    "4": "@hoprnet/hopr-core-connector-interface,1.3.2-35127fb"
  },
  "version": "0.0.1"
}
```

#### Sending & Listening to messages

In this example, you will need to run two servers (server `A` and server `B`), one for sending a message and another for listening.
Server `B` needs to be setup in a different directory from server `A`, as a **HOPR Node** creates a `db` directory which can not be shared between different instances.

1. Start server `A`: `yarn start`
2. Start server `B`: `SERVER_HOST=0.0.0.0:50052 CORE_HOST=0.0.0.0:9092 yarn start`
3. Call `GetStatus` for both servers using BloomRPC, take note of their ids
4. Call `Listen` on server `B`, `peer_id` is optional and can be removed, example input: `{}`
5. Call `Send` on server `A`, example input can be:

```json
{
  "peer_id": "<server B peer ID>",
  "payload": [104, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100]
}
```

6. Server `B` should have received the message as a payload.

### Querying the server using [gCURL](https://github.com/nikunjy/pcurl)

1. Install [gCURL](https://github.com/nikunjy/pcurl) using `npm install -g gcurl` or `yarn global add curl`.
2. Start the server `yarn start`
3. Wait until terminal displays `:: HOPR Core Node Started ::`
4. Call `getStatus` using `gcurl -f ./node_modules/@hoprnet/hopr-protos/protos/status.proto --host 127.0.0.1:50051 --input '{}' --short status:Status:getStatus`
5. First `getStatus` call might take a minute to respond, you should receive a minified json response like:

```json
{
  "id": "16Uiu2HAm6rVeEmviiSoX67H5fqkTQq2ZyP2QSRwrmtEuTU9pWeKj",
  "multi_addresses": [
    "/ip4/93.109.190.135/tcp/9091/p2p/16Uiu2HAm6rVeEmviiSoX67H5fqkTQq2ZyP2QSRwrmtEuTU9pWeKj",
    "/ip4/192.168.178.33/tcp/9091/p2p/16Uiu2HAm6rVeEmviiSoX67H5fqkTQq2ZyP2QSRwrmtEuTU9pWeKj",
    "/ip4/172.17.2.177/tcp/9091/p2p/16Uiu2HAm6rVeEmviiSoX67H5fqkTQq2ZyP2QSRwrmtEuTU9pWeKj",
    "/ip4/127.0.0.1/tcp/9091/p2p/16Uiu2HAm6rVeEmviiSoX67H5fqkTQq2ZyP2QSRwrmtEuTU9pWeKj"
  ],
  "connected_nodes": 11,
  "cpu_usage": 0
}
```

### Considerations

- `BloomRPC` will sometimes insert default input data when calling certain methods, for example with `Send` it will insert:

```json
{
  "peer_id": "316a50f5-4801-4065-bb73-5c602d594ccf",
  "payload": {
    "type": "Buffer",
    "data": [72, 101, 108, 108, 111]
  }
}
```

which is incompatible with our server, the right input is:

```json
{
  "peer_id": "16Uiu2HAm6rVeEmviiSoX67H5fqkTQq2ZyP2QSRwrmtEuTU9pWeKj",
  "payload": [104, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100]
}
```

- Before calling `Send` you should call `GetStatus`, as the **HOPR node** might need to “discover” the other node from the network before it is able to send a message. Avoiding to do so might result in a `Timeout` error.

### Links

- [ ] [License](./LICENSE.md)
- [ ] [Changelog](./CHANGELOG.md)
- [ ] [Contribution Guidelines](./CONTRIBUTING.md)
- [ ] [Issues](./issues)
- [ ] [Codeowners](./CODEOWNERS.md)

<!-- CONTRIBUTORS -->

## Contributors

This project has been possible thanks to the support of the following individuals:

- [@jjperezaguinaga](https://github.com/jjperezaguinaga)
- [@nionis](https://github.com/nionis)
- [@peterbraden](https://github.com/peterbraden)

<!-- CONTACT -->

## Contact

- Twitter - https://twitter.com/hoprnet
- Telegram - https://t.me/hoprnet
- Medium - https://medium.com/hoprnet
- Reddit - https://www.reddit.com/r/HOPR/
- Email - contact@hoprnet.org
- Discord - https://discord.gg/5FWSfq7
- Youtube - https://www.youtube.com/channel/UC2DzUtC90LXdW7TfT3igasA
