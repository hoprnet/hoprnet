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

You can also check out a more detailed overview [here](./doc/protos.md).

## Workflow

1. add a new `.proto` file in `protos` folder
2. running `yarn build` will generate `grpc-node` stubs and `grpc-web` stubs (currenlty web3 stubs are not bundled with their dependancies)
3. you may consum the proto stubs by using:

```javascript
// for node
import { VersionRequest } from '@hoprnet/hopr-protos/node/version_pb'

// for web
import { VersionRequest } from '@hoprnet/hopr-protos/web/version_pb'
```
