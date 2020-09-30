<!-- INTRODUCTION -->
<p align="center">
  <a href="https://hoprnet.org" target="_blank" rel="noopener noreferrer">
    <img width="100" src="https://github.com/hoprnet/hopr-assets/blob/master/v1/logo/hopr_logo_padded.png?raw=true" alt="HOPR Logo">
  </a>
  
  <!-- Title Placeholder -->
  <h3 align="center">HOPR Webapp Demo</h3>
  <p align="center">
    <b>HOPR Webapp Demo</b> is a proof-of-concept web application aimed to showcase the capabilities of a <a href="https://github.com/hoprnet/hopr-core" target="_blank" rel="noopener noreferrer"> HOPR Node</a> by using our gRPC-enabled <a href="https://github.com/hoprnet/hopr-server" target="_blank" rel="noopener noreferrer">
HOPR Server</a>.
  </p>
  <p align="center">
    <b>HOPR Webapp Demo</b> is a <code>web application</code> that allows <code>users</code> to send <code>messages</code> to HOPR nodes available in the network.
  </p>
</p>

<!-- BADGES -->
<p align="center">
  <!-- <a href="#"><img src="https://img.shields.io/static/v1?label=change&message=me&color=yellow" alt="Replace Me"></a> -->
</p>

<!-- TABLE OF CONTENTS -->

## Table of Contents

- [Table of Contents](#table-of-contents)
- [Overview](#overview)
  - [Technologies](#technologies)
- [Getting Started](#getting-started)
  - [Prerequisites](#prerequisites)
  - [Installation](#installation)
- [Usage](#usage)
  - [Environment Variables](#environment-variables)
- [Roadmap](#roadmap)
- [Additional Information](#additional-information)
- [Contributors](#contributors)
- [Contact](#contact)

<!-- OVERVIEW -->

## Overview

**HOPR Webapp Demo** is a chat web application build using [next.js](https://nextjs.org/).

### Technologies

- [next.js](https://github.com/vercel/next.js)
- [typescript](https://www.typescriptlang.org/)
- [grpc-web](https://github.com/grpc/grpc-web)

<!-- GETTING STARTED -->

## Getting Started

To get a local copy up and running follow these simple steps.

### Prerequisites

- [node.js v>=12.9.2](https://nodejs.org/)
- [yarn](https://yarnpkg.com/)
- [docker](https://www.docker.com/) (optional)

### Installation

1. Clone the repo

```sh
git clone https://github.com/hoprnet/hopr-webapp-demo
```

2. Install NPM packages

```sh
yarn
```

<!-- USAGE EXAMPLES -->

## Usage

### Environment Variables

The following environment variables can be stored and used in an `.env` file located at the root of the project.

| Name    | Description                | Type   | Example      |
| :------ | :------------------------- | :----- | :----------- |
| API_URL | hopr-server envoy endpoint | string | 0.0.0.0:8080 |

<!-- ROADMAP -->

## Roadmap

See the [open issues](https://github.com/hoprnet/hopr-webapp-demo/issues) for a list of proposed features (and known issues).

<!-- ADDITIONAL INFORMATION -->

## Additional Information

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

<!-- CONTACT -->

## Contact

- Twitter - https://twitter.com/hoprnet
- Telegram - https://t.me/hoprnet
- Medium - https://medium.com/hoprnet
- Reddit - https://www.reddit.com/r/HOPR/
- Email - contact@hoprnet.org
- Discord - https://discord.gg/5FWSfq7
- Youtube - https://www.youtube.com/channel/UC2DzUtC90LXdW7TfT3igasA
