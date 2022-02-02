---
id: tutorial-hello-world
title: HOPR Apps - Hello world
---

The following is a 5-minutes guide for you to get familiar with the HOPR protocol and start developing apps on top of the
HOPR network.

### Requirements

Before getting started, make sure you have a HOPR cluster available for you to connect. You will need to have the following
environment variables exported in your terminal.

- `apiToken`
- `HOPR_NODE_1_HTTP_URL`
- `HOPR_NODE_1_WS_URL`
- `HOPR_NODE_2_HTTP_URL`
- `HOPR_NODE_2_WS_URL`
- `HOPR_NODE_3_HTTP_URL`
- `HOPR_NODE_3_WS_URL`

If you are using our "Instructions for setting a local HOPR Cluster", you can copy/paste the following Export commands.


<details>
  <summary>Export commands</summary>
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
    <h3>Node 3</h3>
    <br/>
    <pre>
    export HOPR_NODE_3_HTTP_URL=http://127.0.0.1:13303 HOPR_NODE_3_WS_URL=http://127.0.0.1:19503
    </pre>
    <h3>All in one line</h3>
    <br/>
    <pre>
    export apiToken=^^LOCAL-testing-123^^ HOPR_NODE_1_HTTP_URL=http://127.0.0.1:13301 HOPR_NODE_1_WS_URL=http://127.0.0.1:19501 HOPR_NODE_2_HTTP_URL=http://127.0.0.1:13302 HOPR_NODE_2_WS_URL=http://127.0.0.1:19502 HOPR_NODE_3_HTTP_URL=http://127.0.0.1:13303 HOPR_NODE_3_WS_URL=http://127.0.0.1:19503
    </pre>
    </div>
    <br/>
  </div>
</details>

## Connect to multiple nodes

### 1. Install `websocat`

Within our `hoprnet/hoprnet` monorepo, go to the `scripts` directory, and run the `./scripts/install-websocat.sh` script.
After doing it successfully, if you go to the "root" folder of our monorepo, you should be able to run `.bin/websocate` and
get the welcome message.

```
cd scripts
./install-websocat.sh
cd ..
.bin/websocat
```

You can test whether it's working by running `.bin/websocat`

```
websocat 1.8.0
Vitaly "_Vi" Shukela <vi0oss@gmail.com>
Command-line client for web sockets, like netcat/curl/socat for http://.

USAGE:
    websocat http://URL | wss://URL               (simple client)
    websocat -s port                            (simple server)
    websocat [FLAGS] [OPTIONS] <addr1> <addr2>  (advanced mode)

...
```

:::warning @TODO
Modify `install-websocat.sh` so it can be run from `./scripts/install-websocat.sh`
:::

### 2. Connect to your nodes

Using `websocat`, run the following commands, creating a new terminal every time you complete it, and replacing the number
from each URL. Your previously exported variables **might not** persist across terminals, so you will need to run the export
commands again.

**Connecting to node 1**

```
.bin/websocat "$(echo "$HOPR_NODE_1_WS_URL" | sed "s/http/ws/")/?apiToken=$apiToken"
```

**Connecting to node 2**

```
.bin/websocat "$(echo "$HOPR_NODE_2_WS_URL" | sed "s/http/ws/")/?apiToken=$apiToken"
```

**Connecting to node 3**

```
.bin/websocat "$(echo "$HOPR_NODE_3_WS_URL" | sed "s/http/ws/")/?apiToken=$apiToken"
```

## Send messages

The HOPR protocol allows you to send private messages across nodes, by using nodes as relayers. Each message “hops” (hence the
name “HOPR”) until it reaches its final destination, its contents known only to the final recipient.

HOPR nodes can send two types of messages:

- **Multi-hop** messages, which HOPR tokens to be relayed properly, and
- **0-hop** messages, which have no cost, but provide no privacy neither to sender nor the recipient.

### 1. Send a 0-hops

Using `node 2`, type in your terminal the following command:

```
address
```

You should see a response like the following:

```
{"type":"log","msg":"admin > address\n","ts":"2022-02-02T19:17:48.431Z"}
{"type":"log","msg":"HOPR Address:  16Uiu2HAmKhrwGWcvaZ3ic5dgy7oFawmnELJGBrySSsNo4bzGBxHW\nETH Address:   0x4cD95E1deF16D5913255Fe0af208EdDe2e04d720","ts":"2022-02-02T19:17:48.435Z"}
```

As you can see, the address AKA PeerId of `node 2` is `16Uiu2HAmKhrwGWcvaZ3ic5dgy7oFawmnELJGBrySSsNo4bzGBxHW`. You can use that
information to send a message from `node 1`. Within `node 1`, send the following command:

```
send ,16Uiu2HAmKhrwGWcvaZ3ic5dgy7oFawmnELJGBrySSsNo4bzGBxHW Hello world
```

In the terminal of `node 1`, you will see a response similar to this:

```
{"type":"log","msg":"admin > send ,16Uiu2HAmKhrwGWcvaZ3ic5dgy7oFawmnELJGBrySSsNo4bzGBxHW Hello world\n","ts":"2022-02-02T19:19:13.046Z"}
{"type":"log","msg":"Message sent","ts":"2022-02-02T19:19:13.254Z"}
```

and in terminal of `node 2`, you will see something similar to this:

```
{"type":"message","msg":"Hello world","ts":"2022-02-02T19:19:13.233Z"}
```

Congratulations! You have sent your first message using the HOPR protocol!

### 2. Send a 1-hop message to yourself

Multi-hop messages require Payment Channels to be openned between nodes. Your local HOPR cluster was setup in a way that all nodes
have channels openned against each other and funded with testnet HOPR tokens. For a 1-hop message, you will send a packet that contains
a claimable signature for `0.01 HOPR` tokens by the 1st HOP user.

From `node 1`, now let's see a message to ourselves (!) using `node 2` as a relayer. Send the following command to `node 1`:

```
send 16Uiu2HAmKhrwGWcvaZ3ic5dgy7oFawmnELJGBrySSsNo4bzGBxHW,me Its me from the future!
```

Within that same terminal, you should see the following responses:

```
{"type":"log","msg":"Message sent","ts":"2022-02-02T19:24:57.206Z"}
{"type":"log","msg":"#### NODE RECEIVED MESSAGE [2022-02-02T19:24:57.357Z] ####","ts":"2022-02-02T19:24:57.357Z"}
{"type":"log","msg":"Message: Its me from the future!","ts":"2022-02-02T19:24:57.358Z"}
{"type":"log","msg":"Latency: 338ms","ts":"2022-02-02T19:24:57.359Z"}
{"type":"message","msg":"Its me from the future!","ts":"2022-02-02T19:24:57.359Z"}
```

As you can see, the message was received by the same node. You can see `node 2` was used as a relay by running the following command:

```
channels
```

The output is quite verbose, and looks as follows:

```
{"type":"log","msg":"admin > channels\n","ts":"2022-02-02T19:25:59.169Z"}
{"type":"log","msg":"fetching channels...","ts":"2022-02-02T19:25:59.170Z"}
{"type":"log","msg":"\nOutgoing Channel:       0x274c2576adb4912d30754a9d1bca9ed867bd10cdd2ff1a243f66b0ed52014be1\nTo:                     16Uiu2HAm81aTFSEADHXKogxEdoABcptNFbgW3QmeSy5reyUcu9JV\nStatus:                 Open\nBalance:                0.1 txHOPR\n","ts":"2022-02-02T19:25:59.180Z"}
{"type":"log","msg":"\nOutgoing Channel:       0x29a96a57b8335cde160ff4f18ff41b9c48e8689c228ba2169f8d0f6e47c2d85a\nTo:                     16Uiu2HAkw5xy5QnGbruwFwu7ipbFNYoVpFFp5zCqSHYp97rhXUar\nStatus:                 Open\nBalance:                0.1 txHOPR\n","ts":"2022-02-02T19:25:59.181Z"}
{"type":"log","msg":"\nOutgoing Channel:       0x656e0453e8cb927e68f5193ebb2ae3a4e1266d19b1f64cd80db33c34606ad919\nTo:                     16Uiu2HAmTkhzDFBJEZj6etC1QgnHX3Po4FHQnqQudGshadP2uf2w\nStatus:                 Open\nBalance:                0.1 txHOPR\n","ts":"2022-02-02T19:25:59.182Z"}
{"type":"log","msg":"\nOutgoing Channel:       0x9f3abe0290eda936864f8998e71c39d5ed564a989be6b2a16cb8f5ea927ce3d5\nTo:                     16Uiu2HAmKhrwGWcvaZ3ic5dgy7oFawmnELJGBrySSsNo4bzGBxHW\nStatus:                 Open\nBalance:                0.09 txHOPR\n","ts":"2022-02-02T19:25:59.183Z"}
{"type":"log","msg":"\nIncoming Channel:       0x10b4f2de6d13ba48f1cb373c836f7db510001119d14c2b975fa6016ae84e35b2\nFrom:                   16Uiu2HAm81aTFSEADHXKogxEdoABcptNFbgW3QmeSy5reyUcu9JV\nStatus:                 Open\nBalance:                0.1 txHOPR\n","ts":"2022-02-02T19:25:59.188Z"}
{"type":"log","msg":"\nIncoming Channel:       0x9968cd646347f13beb8a7ccb903d6c22442e2d218fa8de4b1a08cecb4285f00c\nFrom:                   16Uiu2HAkw5xy5QnGbruwFwu7ipbFNYoVpFFp5zCqSHYp97rhXUar\nStatus:                 Open\nBalance:                0.1 txHOPR\n","ts":"2022-02-02T19:25:59.189Z"}
{"type":"log","msg":"\nIncoming Channel:       0xadc130545b1d5466fcc3019dcd5fb1a01318f9d3ed1ab2e4cf613e903cb88032\nFrom:                   16Uiu2HAmKhrwGWcvaZ3ic5dgy7oFawmnELJGBrySSsNo4bzGBxHW\nStatus:                 Open\nBalance:                0.11 txHOPR\n","ts":"2022-02-02T19:25:59.190Z"}
{"type":"log","msg":"\nIncoming Channel:       0xffe55f85d5cdb706ddee5a2d470d63c40c9d173421b8fb3f03bd9d83b507f3ba\nFrom:                   16Uiu2HAmTkhzDFBJEZj6etC1QgnHX3Po4FHQnqQudGshadP2uf2w\nStatus:                 Open\nBalance:                0.1 txHOPR\n","ts":"2022-02-02T19:25:59.191Z"}
```

The important lines are these two:

```
{"type":"log","msg":"\nOutgoing Channel:       0x9f3abe0290eda936864f8998e71c39d5ed564a989be6b2a16cb8f5ea927ce3d5\nTo:                     16Uiu2HAmKhrwGWcvaZ3ic5dgy7oFawmnELJGBrySSsNo4bzGBxHW\nStatus:                 Open\nBalance:                0.09 txHOPR\n","ts":"2022-02-02T19:25:59.183Z"}
{"type":"log","msg":"\nIncoming Channel:       0xadc130545b1d5466fcc3019dcd5fb1a01318f9d3ed1ab2e4cf613e903cb88032\nFrom:                   16Uiu2HAmKhrwGWcvaZ3ic5dgy7oFawmnELJGBrySSsNo4bzGBxHW\nStatus:                 Open\nBalance:                0.11 txHOPR\n","ts":"2022-02-02T19:25:59.190Z"}
```

You can see the balance between these two channels have a `+/- 0.01` difference. This means that `node 2` made a profit of `0.01 HOPR`
by relaying your packet. Because `node 2` could only get paid after forwarding the packet successfully, we know nodes are incentivised
to behave properly. This is what we call Proof of Relay.

### 3. Send a 2-hop message to yourself

So what happens if we do `2` hops, i.e. use `2` nodes to relay our messages? First, let's get the address of `node 3`

```
address
```

As before, the output should look like this

```
{"type":"log","msg":"admin > address\n","ts":"2022-02-02T19:30:36.959Z"}
{"type":"log","msg":"HOPR Address:  16Uiu2HAkw5xy5QnGbruwFwu7ipbFNYoVpFFp5zCqSHYp97rhXUar\nETH Address:   0x9d38B703548C0d3025995895184A05ba72e086c6","ts":"2022-02-02T19:30:36.961Z"}
```

Now, from `node 1`, let's send the following command:

```
send 16Uiu2HAmKhrwGWcvaZ3ic5dgy7oFawmnELJGBrySSsNo4bzGBxHW,16Uiu2HAkw5xy5QnGbruwFwu7ipbFNYoVpFFp5zCqSHYp97rhXUar,me 2-hop message, yikes!
```

Again, you should see in your terminal for `node 1` a response similar to this one:

```
{"type":"log","msg":"admin > send 16Uiu2HAmKhrwGWcvaZ3ic5dgy7oFawmnELJGBrySSsNo4bzGBxHW,16Uiu2HAkw5xy5QnGbruwFwu7ipbFNYoVpFFp5zCqSHYp97rhXUar,me 2-hop message, yikes!\n","ts":"2022-02-02T19:31:56.106Z"}
{"type":"log","msg":"Message sent","ts":"2022-02-02T19:31:56.282Z"}
{"type":"log","msg":"#### NODE RECEIVED MESSAGE [2022-02-02T19:31:56.540Z] ####","ts":"2022-02-02T19:31:56.540Z"}
{"type":"log","msg":"Message: 2-hop message, yikes!","ts":"2022-02-02T19:31:56.541Z"}
{"type":"log","msg":"Latency: 425ms","ts":"2022-02-02T19:31:56.542Z"}
{"type":"message","msg":"2-hop message, yikes!","ts":"2022-02-02T19:31:56.542Z"}
```

The only difference now will be when you run the `channels` command again:

```
{"type":"log","msg":"admin > channels\n","ts":"2022-02-02T19:33:13.081Z"}
{"type":"log","msg":"fetching channels...","ts":"2022-02-02T19:33:13.081Z"}
{"type":"log","msg":"\nOutgoing Channel:       0x274c2576adb4912d30754a9d1bca9ed867bd10cdd2ff1a243f66b0ed52014be1\nTo:                     16Uiu2HAm81aTFSEADHXKogxEdoABcptNFbgW3QmeSy5reyUcu9JV\nStatus:                 Open\nBalance:                0.1 txHOPR\n","ts":"2022-02-02T19:33:13.093Z"}
{"type":"log","msg":"\nOutgoing Channel:       0x29a96a57b8335cde160ff4f18ff41b9c48e8689c228ba2169f8d0f6e47c2d85a\nTo:                     16Uiu2HAkw5xy5QnGbruwFwu7ipbFNYoVpFFp5zCqSHYp97rhXUar\nStatus:                 Open\nBalance:                0.1 txHOPR\n","ts":"2022-02-02T19:33:13.094Z"}
{"type":"log","msg":"\nOutgoing Channel:       0x656e0453e8cb927e68f5193ebb2ae3a4e1266d19b1f64cd80db33c34606ad919\nTo:                     16Uiu2HAmTkhzDFBJEZj6etC1QgnHX3Po4FHQnqQudGshadP2uf2w\nStatus:                 Open\nBalance:                0.1 txHOPR\n","ts":"2022-02-02T19:33:13.096Z"}
{"type":"log","msg":"\nOutgoing Channel:       0x9f3abe0290eda936864f8998e71c39d5ed564a989be6b2a16cb8f5ea927ce3d5\nTo:                     16Uiu2HAmKhrwGWcvaZ3ic5dgy7oFawmnELJGBrySSsNo4bzGBxHW\nStatus:                 Open\nBalance:                0.07 txHOPR\n","ts":"2022-02-02T19:33:13.098Z"}
{"type":"log","msg":"\nIncoming Channel:       0x10b4f2de6d13ba48f1cb373c836f7db510001119d14c2b975fa6016ae84e35b2\nFrom:                   16Uiu2HAm81aTFSEADHXKogxEdoABcptNFbgW3QmeSy5reyUcu9JV\nStatus:                 Open\nBalance:                0.1 txHOPR\n","ts":"2022-02-02T19:33:13.106Z"}
{"type":"log","msg":"\nIncoming Channel:       0x9968cd646347f13beb8a7ccb903d6c22442e2d218fa8de4b1a08cecb4285f00c\nFrom:                   16Uiu2HAkw5xy5QnGbruwFwu7ipbFNYoVpFFp5zCqSHYp97rhXUar\nStatus:                 Open\nBalance:                0.1 txHOPR\n","ts":"2022-02-02T19:33:13.108Z"}
{"type":"log","msg":"\nIncoming Channel:       0xadc130545b1d5466fcc3019dcd5fb1a01318f9d3ed1ab2e4cf613e903cb88032\nFrom:                   16Uiu2HAmKhrwGWcvaZ3ic5dgy7oFawmnELJGBrySSsNo4bzGBxHW\nStatus:                 Open\nBalance:                0.13 txHOPR\n","ts":"2022-02-02T19:33:13.110Z"}
{"type":"log","msg":"\nIncoming Channel:       0xffe55f85d5cdb706ddee5a2d470d63c40c9d173421b8fb3f03bd9d83b507f3ba\nFrom:                   16Uiu2HAmTkhzDFBJEZj6etC1QgnHX3Po4FHQnqQudGshadP2uf2w\nStatus:                 Open\nBalance:                0.1 txHOPR\n","ts":"2022-02-02T19:33:13.111Z"}
```

The balance is now quite different! As before, we have to look at these two lines:

```
{"type":"log","msg":"\nOutgoing Channel:       0x9f3abe0290eda936864f8998e71c39d5ed564a989be6b2a16cb8f5ea927ce3d5\nTo:                     16Uiu2HAmKhrwGWcvaZ3ic5dgy7oFawmnELJGBrySSsNo4bzGBxHW\nStatus:                 Open\nBalance:                0.07 txHOPR\n","ts":"2022-02-02T19:33:13.098Z"}
{"type":"log","msg":"\nIncoming Channel:       0xadc130545b1d5466fcc3019dcd5fb1a01318f9d3ed1ab2e4cf613e903cb88032\nFrom:                   16Uiu2HAmKhrwGWcvaZ3ic5dgy7oFawmnELJGBrySSsNo4bzGBxHW\nStatus:                 Open\nBalance:                0.13 txHOPR\n","ts":"2022-02-02T19:33:13.110Z"}
```

This time we see that the difference is for `0.02 HOPR` tokens. These tokens were included in your message, and used as payment
for each of the hops made in the network.

That's it! You are ready to start sending messages and building applications on top of the HOPR protocol. If you want to learn
how to build a simple Web3 application, see the next section.

### HOPR Admin UI and REST API

We ran all these commands via our WebSocket API, but you can also see them via our Web UI interface called `hopr-admin`.
In a browser, you can simply paste your `HOPR_NODE_1_WS_URL`. You should be able to see an image like the following one:

_HOPR Admin Image_

Likewise, if you want to see what is currently supported in our API, you can see the Swagger UI in `HOPR_NODE_1_HTTP_URL/api/v2/_swagger/`.


### Walkthrough

In case you need some help to complete this tutorial, you can watch our 15-minute walkthrough which includes also the setup
of the local HOPR cluster.

_Video_