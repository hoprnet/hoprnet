# HOPR

HOPR is a privacy-preserving messaging **protocol** which enables the creation of a secure communication network via relay nodes powered by economic incentives using digital tokens.

## Testing

Testing is done by trying to generate proto stubs for node and web, if building fails then it means something is wrong with our protos, ideally, we can improve this to also include linting and breaking change detection.

## Protos Architecture

```
Stage 0 - describe        GET   /status ({} => { id, multi_addresses, cpu_usage, connected_nodes })
Stage 0 - describe        GET   /version ({} => { version, components_version })
Stage 0 - quit            POST  /shutdown ({} => { timestamp })
Stage 0 - ping            POST  /ping ({peerId} => { pingReceipt:latency })
Stage 0 - .env            POST  /settings ({Settings:bootstrap_servers, is_using_cover_traffic})

Stage 1 - balance         GET   /balance/native ({} => { amount })
Stage 1 - balance         GET   /balance/hopr ({} => { amount })
Stage 1 - myAddress       GET   /address/native ({} => { amount })
Stage 1 - myAddress       GET   /address/hopr ({} => { amount })

Stage 2.a - listChannel   GET   /channels ({} => { open_channel[] })
Stage 2.a - openChannel   POST  /channels ({peerId} => channelTxReceipt:{channelId,...})
Stage 2.a - listChannel   GET   /channels/:channelId ({channelId} => { state, balance, ... })
Stage 2.a - closeChannel  POST  /channels/:channelId/close ({channelId} => channelTxReceipt)
Stage 2.b - crawl         POST  /crawl ({} => connected_nodes[])
Stage 2.b - listen*       POST  /listen ({} => EventHandler) // e.g. on.message(..., fn)
Stage 2.b - send          POST  /send ({peerID, payload, [intermediatePeerIds[], timeout]} => txReceipt)

Stage 3 - transfer*       POST  /transfer/native ({address, amount} => txReceipt)
Stage 3 - transfer*       POST  /transfer/hopr ({address,amount} => txReceipt)
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
