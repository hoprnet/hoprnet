# HOPR

HOPR is a privacy-preserving messaging **protocol** which enables the creation of a secure communication network via relay nodes powered by economic incentives using digital tokens.

## Testing

Testing is done by trying to generate proto stubs for node and web, if building fails then it means something is wrong with our protos, ideally, we can improve this to also include linting and breaking change detection.

## Protos Architecture

```
Stage 0 - describe        GET   /status ({} => { id, multiAddresses, connectedNodes })
Stage 0 - describe        GET   /version ({} => { version, componentsVersion })
Stage 0 - quit            POST  /shutdown ({} => { timestamp })
Stage 0 - ping            POST  /ping ({peerId} => { latency })
Stage 0 - .env            POST  /settings ({ bootstrapServers, isUsingCoverTraffic })

Stage 1 - balance         GET   /balance/native ({} => { amount })
Stage 1 - balance         GET   /balance/hopr ({} => { amount })
Stage 1 - myAddress       GET   /address/native ({} => { amount })
Stage 1 - myAddress       GET   /address/hopr ({} => { amount })

Stage 2.a - listChannel   GET   /channels ({} => { openChannel[] })
Stage 2.a - openChannel   POST  /channels ({ peerId } => { channelId, txHash })
Stage 2.a - listChannel   GET   /channels/:channelId ({ channelId } => { state, balance, ... })
Stage 2.a - closeChannel  POST  /channels/:channelId/close ({ channelId  } => { txHash })
Stage 2.b - listen*       POST  /listen ({ [peerId] } => { stream:payload })
Stage 2.b - send          POST  /send ({ peerId, payload, [intermediatePeerIds[], timeout]} => { intermediatePeerIds[] })

Stage 3 - transfer*       POST  /transfer/native ({ address,  amount } => { txHash })
Stage 3 - transfer*       POST  /transfer/hopr ({ address, amount } => { txHash })
```

You can also check out a more detailed overview [here](./docs/protos.md).

## Workflow

1. add a new `.proto` file in `protos` folder
2. running `yarn build` will generate [grpc-node](https://github.com/grpc/grpc-node/tree/grpc@1.24.x/packages/grpc-native-core) stubs and [grpc-web](https://github.com/grpc/grpc-web) stubs

example:

```javascript
// for node
import { VersionRequest } from '@hoprnet/hopr-protos/node/version_pb'

// for web
import { VersionRequest } from '@hoprnet/hopr-protos/web/version_pb'
```

## Generating docs

You can take a look at [generateDocs](./scripts/generateDocs.sh) script on how to generate docs.

## Notes on @grpc/grpc-js

Eventually we will have to switch to [@grpc/grpc-js](https://github.com/grpc/grpc-node/tree/master/packages/grpc-js) since [@grpc/grpc-native-core](https://github.com/grpc/grpc-node/tree/grpc@1.24.x/packages/grpc-native-core) is being [deprecated](https://github.com/grpc/grpc-node/blob/master/PACKAGE-COMPARISON.md).

We can do that once these issues are closed [1](https://github.com/nestjs/nest/issues/4799), [2](https://github.com/improbable-eng/ts-protoc-gen/issues/226).

## Gotchas

- web stubs are not bundled with their dependancies
