---
id: connecting-node
title: Interacting with a HOPR node
---

Once you have your HOPR cluster up and running, you are ready to connect to a single HOPR node and listen to messages sent to it by other HOPR nodes.
The following guide will show you how to connect to a running HOPR node, and verify some basic functionality using both the REST and WebSocket
endpoints.

## Connect to a HOPR node

### 1. Get the node security credentials

To avoid unsecured access to your HOPR node, all HOPR node's WebSocket and REST calls require an `apiToken`. Your API token needs to be appended as a
`query` parameter for WebSocket connections and as encoded token via the `Authorization` header for you to be able to connect to it.

:::info Tip
The flag used to set this value in a HOPR node via `hoprd` is `--apiToken`. The default `apiToken` used across our documentation is the following one 
(click on the right to copy), so make sure to change it when running your own node in a public network.
<br/>

```
^^LOCAL-testing-123^^
```
:::

Export your `apiToken` to be used in the incoming commands.

```
export apiToken="^^LOCAL-testing-123^^"
```

### 2. Test REST API connectivity

Given your node's `apiToken` and your `HOPR_NODE_1_HTTP_URL` from the last section, try to send a REST command to query it’s address with the following `curl`
command. If you don’t have `jq` installed, just remove it at the end of the command.

```
echo -n $apiToken | base64 | xargs -I {} curl -s -H "Authorization: Basic {}" $HOPR_NODE_1_HTTP_URL/api/v2/account/address | jq
```

If successful, you should get a response similar to this one:

```
{
  "nativeAddress": "0x3a54dDE3ee5ACfd43C902cbecC8ED0CBA10Ff326",
  "hoprAddress": "16Uiu2HAmE9b3TSHeF25uJS1Ecf2Js3TutnaSnipdV9otEpxbRN8Q"
}
```

In case you have made a mistake, like forgotten to use `-n` in your `echo` or have the wrong `apiToken`, you’ll see the following instead:

```
{
  "status": 403,
  "challenge": "Basic realm=hoprd",
  "message": "You must authenticate to access hoprd."
}
```

### 3. Test WebSocket connectivity

Unlike our REST API endpoint, to see interactions against our HOPR node WebSocket server, we need to use a client to both send and listen
connections in our HOPR node. Our suggested client is [websocat](https://github.com/vi/websocat), which you can install by running our
`./scripts/install-websocat.sh` [script](https://raw.githubusercontent.com/hoprnet/hoprnet/master/scripts/install-websocat.sh) from our monorepo.

:::warning @TODO
Migrate `install-websocat.sh` script to allow one-line setup via bash pipeing
```
curl -o- https://raw.githubusercontent.com/hoprnet/hoprnet/master/scripts/install-websocat.sh | bash
```
:::

With `websocat` installed, run the following command to connect to your HOPR node WebSocket server. Please pay attention that we are now using
our `HOPR_NODE_1_WS_URL` instead of the `HOPR_NODE_1_HTTP_URL` from last step, which is also referred as `Admin URL` in our tools.

```
.bin/websocat "$(echo "$HOPR_NODE_1_WS_URL" | sed "s/http/ws/")/?apiToken=$apiToken"
```

If worked correctly, you should see a dump of messages, the last one being:

```
{"type":"log","msg":"ws client connected [ authentication ENABLED ]","ts":"2022-02-01T19:42:34.152Z"}
```

Now that you are connected, try typing `balance` in the same terminal, which should output as follows:

```
{"type":"log","msg":"admin > balance\n","ts":"2022-02-01T19:42:35.417Z"}
{"type":"log","msg":"HOPR Balance:  9.6 txHOPR\nETH Balance:   0.99871794476851171 xDAI","ts":"2022-02-01T19:42:35.421Z"}
```

With the connection verified to both our REST and WebSocket endpoints, we can now go ahead and go through the basic functions of the API to send
messages across nodes.

## Additional REST/WebSocket clients

On top of `curl` and `websocat`, the following clients can also help you debug your HOPR node to ensure the API is working properly.
Be aware that you will still need to know your `apiToken`.

**WebSocket clients**

- [Piesocket WebSocket Tester](https://www.piesocket.com/websocket-tester): This is a great tool to debug both listening and sending to
messages from/to your HOPR node. Make sure you paste your `HOPR_NODE_1_WS_URL` and append your `apiToken` as a query parameter. Also,
you need to change the `http` protocol to `ws`. For instance, here's how this would look like in a `Gitpod.io` instance. After it's
connected, you can type `balance` to see your node response.

```
wss://19501-hoprnet-mynechat-7x6h2ghc17f.ws-us30.gitpod.io/?apiToken=^^LOCAL-testing-123^^
```

**REST client**

- [ReqBin](https://reqbin.com/): Using their `Custom` Header option, you can send the proper `Authorization` request so you can test your
HOPR node endpoint. For testing, we suggest using `HOPR_NODE_1_HTTP_URL` and the `api/v2/account/address` endpoint. Make sure sure to use
the `base64` encoded version of your `apiToken` and adding the prefix `Basic `.

:::info Tip
For `apiToken` `^^LOCAL-testing-123^^` the `base64` encoded value is `Xl5MT0NBTC10ZXN0aW5nLTEyM15e`, so to use [ReqBin](https://reqbin.com/)
with a Gitpod exposed URL, you can do the following. For a different `apiToken` value, you can use the `btoa` function of your browser
Developer Tools to figure it out.

<br/>

**URL**
```
https://13302-hoprnet-mynechat-7x6h2ghc17f.ws-us30.gitpod.io/api/v2/account/address
```

**Custom Header**
```
Basic Xl5MT0NBTC10ZXN0aW5nLTEyM15e
```
:::
