# HOPR

HOPR is a privacy-preserving messaging **protocol** which enables the creation of a secure communication network via relay nodes powered by economic incentives using digital tokens.

## Query the server using [gCURL](https://github.com/nikunjy/pcurl)

1. Install [gCURL](https://github.com/nikunjy/pcurl) using `npm install -g gcurl`
2. Clone this project `git clone https://github.com/hoprnet/hopr-core.git`
3. Go to `protos` folder `cd protos`
4. Install dependancies using `yarn`
5. Start GRPC server `yarn start`
6. Wait until temrinal displays `HOPR Core Node Started`
7. Call `getStatus` using `gcurl -f ./node_modules/@hoprnet/hopr-protos/protos/status.proto --host 127.0.0.1:50051 --input '{}' --short status:Status:getStatus`
8. First `getStatus` call might take a minute to respond, you should receive a minified json response like:

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
