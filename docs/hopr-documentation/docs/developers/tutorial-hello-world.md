---
id: tutorial-hello-world
title: HOPR Apps - Hello world
---

The following is a 5-minutes guide for you to get familiar with the HOPR protocol and start developing apps on top of the
HOPR network by learning how to send and read messages from one node to another.

## Requirements

### 1. Previous guides

Before getting started, we suggest you get familiar with the following concepts:

- ["Running a local HOPR Cluster"](/developers/starting-local-cluster)
- ["Interacting with a HOPR node"](/developers/connecting-node)

### 2. HOPR Cluster & exported variables

Make sure you have a HOPR cluster[^1] available for you to connect. Also, make sure to have the equivalent of at least `2`
nodes endpoints (both HTTP and WS). Here are the default values for each of these endpoints.

<details>
  <summary>Default values for <code>apiToken</code> and nodes endpoints</summary>
  <div>
    <div>
    <h3>apiToken</h3>
    <pre>
    ^^LOCAL-testing-123^^
    </pre>
    <h3>HOPR_NODE_1_HTTP_URL</h3>
    <pre>
    http://127.0.0.1:13301
    </pre>
    <h3>HOPR_NODE_1_WS_URL</h3>
    <pre>
    http://127.0.0.1:19501
    </pre>
    <h3>HOPR_NODE_2_HTTP_URL</h3>
    <pre>
    http://127.0.0.1:13302
    </pre>
    <h3>HOPR_NODE_2_WS_URL</h3>
    <pre>
    http://127.0.0.1:19502
    </pre>
    </div>
  </div>
</details>

We'll assume your HOPR nodes had been run with "Instructions for setting a local HOPR Cluster". If you are looking to
interact with your node via a terminal, we suggest you to copy/paste the following commands every time you open a new
terminal.

<details>
  <summary><code>export</code> commands for terminal</summary>
  <div>
    <div>
    <h3>API Token</h3>
    <br/>
    <pre>
    export apiToken=^^LOCAL-testing-123^^
    </pre>
    <h3>Node 1</h3>
    <br/>
    <pre>
    export HOPR_NODE_1_HTTP_URL=http://127.0.0.1:13301 HOPR_NODE_1_WS_URL=http://127.0.0.1:19501
    </pre>
    <h3>Node 2</h3>
    <br/>
    <pre>
    export HOPR_NODE_2_HTTP_URL=http://127.0.0.1:13302 HOPR_NODE_2_WS_URL=http://127.0.0.1:19502
    </pre>
    <h3>All in one line</h3>
    <br/>
    <pre>
    export apiToken=^^LOCAL-testing-123^^ HOPR_NODE_1_HTTP_URL=http://127.0.0.1:13301 HOPR_NODE_1_WS_URL=http://127.0.0.1:19501 HOPR_NODE_2_HTTP_URL=http://127.0.0.1:13302 HOPR_NODE_2_WS_URL=http://127.0.0.1:19502
    </pre>
    </div>
    <br/>
  </div>
</details>

## Connect to your nodes

### 1. Connect to the WebSocket server of `node 2`

Using `websocat` or any other WebSocket client, connect to your `node 2` until you are able to receive and send messages to it.

**Connecting to `node 2` via `websocat`**

```
.bin/websocat "$(echo "$HOPR_NODE_2_WS_URL" | sed "s/http/ws/")/?apiToken=$apiToken"
```

**Connecting to `node 2` via [Piesocket WebSocket Tester](https://www.piesocket.com/websocket-tester)**

```
ws://127.0.0.1:19502/?apiToken=^^LOCAL-testing-123^^
```

You can verify that you are connected by typing the command `address` and seeing an output similar to this:

```
{"type":"log","msg":"admin > address\n","ts":"2022-02-02T19:17:48.431Z"}
{"type":"log","msg":"HOPR Address:  16Uiu2HAmKhrwGWcvaZ3ic5dgy7oFawmnELJGBrySSsNo4bzGBxHW\nETH Address:   0x4cD95E1deF16D5913255Fe0af208EdDe2e04d720","ts":"2022-02-02T19:17:48.435Z"}
```

### 2. Verify REST API connectivity for `node 1`

Using `curl` or any other HTTP client, verify you can reach your `node 1`'s API

**Obtaining the `node 1` address using `curl`**

```
echo -n $apiToken | base64 | xargs -I {} curl -s -H "Authorization: Basic {}" $HOPR_NODE_1_HTTP_URL/api/v2/account/address | jq
```

**Obtaining the `node 1` address using [reqbin](https://reqbin.com/)**

_URL_

```
http://127.0.0.1:13301/api/v2/account/address
```

_Custom Header (default `apiToken` `base64`-encoded)_

```
Basic Xl5MT0NBTC10ZXN0aW5nLTEyM15e
```

If you sent a successful request, the response will look something like this:

```
{
  "nativeAddress": "0x3a54dDE3ee5ACfd43C902cbecC8ED0CBA10Ff326",
  "hoprAddress": "16Uiu2HAmE9b3TSHeF25uJS1Ecf2Js3TutnaSnipdV9otEpxbRN8Q"
}
```

## Send messages

The HOPR protocol allows you to send private messages across nodes, by using nodes as relayers. Each message “hops” (hence the
name “HOPR”) until it reaches its final destination, its contents known only to the final recipient. This is done automatically
by HOPR nodes, which pick a random path to send your message to your recipient.

The path consists of at least `2` nodes that have channels openned between each others. For instance, to send a message to `node 2`
from `node 1`, the path a message could take is `node 1 -> node 4 -> node 3 -> node 2`.

We'll try doing this, we'll use the REST API from `node 1` and send a message to `node 2`, which we'll be able to see via our
WebSocket client connection.

### 1. Obtaining the address (`PeerId`) of `node 2`

Using `node 2`, type in your terminal with `websocat` running or WebSocket client interface the following command:

```
address
```

You should see a response like the following:

```
{"type":"log","msg":"admin > address\n","ts":"2022-02-02T19:17:48.431Z"}
{"type":"log","msg":"HOPR Address:  16Uiu2HAmKhrwGWcvaZ3ic5dgy7oFawmnELJGBrySSsNo4bzGBxHW\nETH Address:   0x4cD95E1deF16D5913255Fe0af208EdDe2e04d720","ts":"2022-02-02T19:17:48.435Z"}
```

As you can see, the address AKA PeerId of `node 2` is `16Uiu2HAmKhrwGWcvaZ3ic5dgy7oFawmnELJGBrySSsNo4bzGBxHW`. You can use that
information to send a message from `node 1`. Make sure to keep your WebSocket client connected to see any messages to `node 2`

### 2. Sending a message from `node 2` from `node 1`:

To send a message from `node 1` to `node 2`, we need to use `node 1`'s REST API, particularly the `/messages` endpoint. Using `curl`
or any other HTTP client, send the following request:

```
echo -n $apiToken | base64 | xargs -I {} curl -s -H "Authorization: Basic {}" \
-H 'Content-Type: application/json' \
-d '{"body":"Hello world","recipient":"16Uiu2HAmKhrwGWcvaZ3ic5dgy7oFawmnELJGBrySSsNo4bzGBxHW"}' \
$HOPR_NODE_1_HTTP_URL/api/v2/messages
```

In the terminal of `node 2`, you will see something similar to this:

```
{"type":"log","msg":"#### NODE RECEIVED MESSAGE [2022-02-03T21:48:13.845Z] ####","ts":"2022-02-03T21:48:13.845Z"}
{"type":"log","msg":"Message: Hello world","ts":"2022-02-03T21:48:13.846Z"}
{"type":"log","msg":"Latency: 668ms","ts":"2022-02-03T21:48:13.846Z"}
{"type":"message","msg":"Hello world","ts":"2022-02-03T21:48:13.847Z"}
```

Congratulations! You have sent your first message using the HOPR protocol!

### 3. Extra: Channels and tickets

You can not send an unlimited amount of messages[^2]. Each message requires `ticket`s, headers with signatures for claiming
balance updates in a `HOPR` token balanced Payment Channel. For every relayer you use (default `2`) you need to "attach"
`HOPR` tokens (`0.01` per relay), which are used to pay for relayer's work, and are settled in a global single entry
`HoprChannels` Ethereum contract.

The previous message worked because a cluster has been configured by default to open a few channels and locked enough `HOPR`
tokens to send messages to at least `2` relayers. When a path used to relay has depleted (empty) or closed `channels`,
your message will no longer be forwarded. You can always see your open `channels` and ther balance with the following command.

```
channels
```

Likewise, you can see your balance via the following command.

```
balance
```

## HOPR Admin UI and REST API

We ran all these commands via our WebSocket API, but you can also see them via our Web UI interface called `hopr-admin`.
In a browser, you can simply paste your `HOPR_NODE_1_WS_URL`. You should be able to see an image like the following one:

![HOPR Admin Image](/img/developer/hopr_admin_ui.png)

## Walkthrough

In case you need some help to complete this tutorial, you can watch our 15-minute walkthrough which includes also the setup
of the local HOPR cluster.

<figure class="video-container" style={{"marginTop": "-100px", "marginBottom": "-100px"}}>
  <iframe src="https://player.vimeo.com/video/672847960?h=bc02050298" width="640" height="564" frameborder="0" allow="autoplay; fullscreen" allowfullscreen></iframe>
</figure>

[^1]:
    As defined in ["Running a local HOPR Cluster"](/developers/starting-local-cluster), a HOPR cluster is a set of HOPR nodes that are connected (i.e
    reachable) between each other and have openned funded channels between each other. This is also known as a [Complete
    Graph](https://en.wikipedia.org/wiki/Complete_graph), and although it is not required for successfully sending a message
    (i.e. you only need `1` path to a recipient when sending a message), it makes testing and developing HOPR apps easier.

[^2]:
    When a HOPR node uses the `/api/v2/messages` REST API endpoint to send a message, it will use a random path of at
    least `2` nodes as relayers. These messages cost `$HOPR` tokens, because it pays each relayer accordingly to forward the
    packet. However, you can also send what it is known as `0-hop` messages, which bears no cost, but provides no privacy, as
    the message is sent directly to the recipient. The REST API endpoint does not expose this, but you can send a `0-hop`
    message via the HOPR admin UI or WebSocket client by typing `send ,$peerId message`.
