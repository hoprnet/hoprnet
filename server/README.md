# HOPR

HOPR is a privacy-preserving messaging **protocol** which enables the creation of a secure communication network via relay nodes powered by economic incentives using digital tokens.

## Installing

1. Clone repository: `git clone https://github.com/hoprnet/hopr-core.git`
2. Navigate to the server's subdirectory: `cd server`
3. Install dependancies: `yarn`

## Running and stopping the server

1. Start server: `yarn start`
2. Once you see `:: HOPR Core Node Started ::`, it means that the server is ready to accept requests.

To `stop` the server, you need press `ctrl+c` on your terminal window, this will stop the server gracefully.

## Environment Variables

The server is able to recognise and process a `.env` file located at the root of the project.

| Name              | Description                                                  | Type             | Example                           |
| :---------------- | :----------------------------------------------------------- | :--------------- | :-------------------------------- |
| SERVER_HOST       | server HOST url                                              | string           | 0.0.0.0:50051                     |
| DEBUG             | passed to hopr-core: run in debug mode                       | boolean          | TRUE                              |
| ID                | passed to hopr-core: demo account ID                         | integer          | 1                                 |
| BOOTSTRAP_NODE    | passed to hopr-core: TRUE if node is a boostrap node         | boolean          | FALSE                             |
| CORE_HOST         | passed to hopr-core: hopr-core HOST url                      | string           | 0.0.0.0:9091                      |
| BOOTSTRAP_SERVERS | passed to hopr-core: a list of bootstap server to connect to | array of strings | [src](./src/core/core.service.ts) |

## Query the server using [BloomRPC](https://github.com/uw-labs/bloomrpc) (Recommended)

1. Download and install [BloomRPC](https://github.com/uw-labs/bloomrpc/releases)
2. BloomRPC requires us to:
   1. import the `.proto` files that are used by our server
   2. set our server url `127.0.0.1:50051` so it knows where to send requests to
3. import `.proto` files by click on the top-left "+" icon and navigating to `node_modules/@hoprnet/hopr-protos/protos` select all `.proto` files
4. after importing the available methods should be visible on the left panel
5. set server url in the input at top-center to `127.0.0.1:50051`

### Quering

1. Select on of the unary methods from the left panel, for example: `version.proto -> version.Version -> GetVersion`
2. Click "play" (▶️), you should get a response of something like:

```json
{
  "components_version": {
    "0": "@hoprnet/hopr-core,0.0.9-97c8c3e",
    "1": "@hoprnet/hopr-core-connector-interface,",
    "2": "@hoprnet/hopr-core-ethereum,",
    "3": "@hoprnet/hopr-utils,",
    "4": "@hoprnet/hopr-core-connector-interface,"
  },
  "version": "0.0.1"
}
```

### Sending & Listening to messages

In this example, you will need to run two servers (server A and server B), one for sending a message and another for listening.
Server B should be setup in a different directory from server A.

1. Start server A: `yarn start`
2. Start server B: `SERVER_HOST=0.0.0.0:50052 CORE_HOST=0.0.0.0:9092 yarn start`
3. Call `GetStatus` for both servers using BloomRPC, take note of their ids
4. Call `Listen` on server B, `peer_id` is optional and can be removed, example input: `{}`
5. Call `Send` on server A, example input can be:

```json
{
  "peer_id": "<server B peer ID>",
  "payload": [104, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100]
}
```

6. Server B should have received a message

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

## Gotchas

- `BloomRPC` will sometime insert default input data when calling certain methods, for example with `Send` it will insert:
```json
{
  "peer_id": "316a50f5-4801-4065-bb73-5c602d594ccf",
  "payload": {
    "type": "Buffer",
    "data": [
      72,
      101,
      108,
      108,
      111
    ]
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

- First `GetStatus` might take a long time respond, this is because internally we do a `crawl`, tracking issue [here](https://github.com/hoprnet/hopr-core/issues/156).

- Before calling `Send` you should call `GetStatus`