---
id: connecting-node
title: Interacting with a HOPR node
---

Once you have your HOPR cluster up and running, you are ready to connect to a single HOPR node and listen to messages sent to it by other HOPR nodes.
The following guide will show you how to connect to a running HOPR node, and verify some basic functionality using both the REST and WebSocket
endpoints.

## Requirements

### 1. Get the node security credentials

To avoid unsecured access to your HOPR node, all HOPR node's WebSocket and REST calls require an `apiToken`. Your API token needs to be appended as a
`query` parameter for WebSocket connections and as an encoded token via the `Authorization` header for you to be able to connect to it. You will not be
able to connect to your HOPR node without its `apiToken`.

:::info Tip
The flag used to set this value in a HOPR node via `hoprd` is `--apiToken`. The default `apiToken` used across our documentation is the following one,
so make sure to change it when running your own node in a public network.
<br/>

```
^^LOCAL-testing-123^^
```

:::

Make sure to export your `apiToken` to be used in the incoming commands, and every time you open a new terminal.

```bash
export apiToken="^^LOCAL-testing-123^^"
```

### 2. Export your HOPR node REST/WebSocket endpoints

If you followed the guide in the ["HOPR Cluster Development Setup"](/developers/starting-local-cluster) section, these will be already exported in your current terminal. Otherwise, you can run the following commands to ensure at least your first node's endpoints are exported.

As an alternative, you an also run a single HOPR node following our [monorepo](https://github.com/hoprnet/hoprnet#develop) instructions.

<details>
  <summary>Export REST and WebSocket endpoints (from local HOPR cluster)</summary>
  <div>
    <div>
    <h3>Node 1</h3>
    <br/>
    <pre>
    export HOPR_NODE_1_HTTP_URL=http://127.0.0.1:13301 HOPR_NODE_1_WS_URL=ws://127.0.0.1:19501
    </pre>
    <h3>API token & Node 1</h3>
    <br/>
    <pre>
    export apiToken=^^LOCAL-testing-123^^ HOPR_NODE_1_HTTP_URL=http://127.0.0.1:13301 HOPR_NODE_1_WS_URL=ws://127.0.0.1:19501
    </pre>
    </div>
    <br/>
  </div>
</details>

<details>
  <summary>Export REST and WebSocket endpoints (from a single localhost HOPR node)</summary>
  <div>
    <div>
    <h3>Node 1</h3>
    <br/>
    <pre>
    export HOPR_NODE_1_HTTP_URL=http://127.0.0.1:3001 HOPR_NODE_1_WS_URL=ws://127.0.0.1:3000
    </pre>
    <h3>API token & Node 1</h3>
    <br/>
    <pre>
    export apiToken=^^LOCAL-testing-123^^ HOPR_NODE_1_HTTP_URL=http://127.0.0.1:3001 HOPR_NODE_1_WS_URL=ws://127.0.0.1:3000
    </pre>
    </div>
    <br/>
  </div>
</details>

We use `127.0.0.1` instead of `localhost` as some tools like `websocat` (described below) struggle to resolve `localhost` properly.

### 3. Install a WebSocket and REST client

To properly[^1] interact with a HOPR node, you'll need both REST and WebSocket client software. A REST client is used to give instructions to your
HOPR node via its REST API, and the WebSocket client is to listen to these interactions, usually given by other nodes.

We recommend using `curl` and `websocat` to interact with both endpoints. These tools are terminal only and supported by any `Unix`-based OS.
If you would like to use a UI-based alternative, please check the [Additional REST/WebSocket clients](#additional-restwebsocket-clients) section.
For the purposes of this guide, `curl` and `websocat` will be assumed.

<details>
  <summary>Installing curl</summary>
  <div>
    <div>Most <code>Unix</code>-based systems already have <code>curl</code> installed, but if you don't have it you can always use the default package manager to do so. For instance, here’s how you install `curl` in Ubuntu:

<pre>
sudo apt-get install curl
</pre>

You can see if <code>curl</code> is installed in your system by running <code>which curl</code> or simply running <code>curl</code>, which will output a message like the following:

<pre>
curl: try 'curl --help' or 'curl --manual' for more information
</pre>
  </div>
  </div>
</details>

<details>
  <summary>Installing websocat</summary>
  <div>
    <div>
    <p>Our suggested client is <a href="https://github.com/vi/websocat" target="_blank" noreferral>websocat</a>, which you can install by running our
<code>./install-websocat.sh</code> <a href="https://raw.githubusercontent.com/hoprnet/hoprnet/master/scripts/install-websocat.sh" taget="_blank" noreferral>script</a> from our monorepo. To install, make sure to run it from the <code>scripts</code> folder, as by default it will install it in the parent folder under a <code>.bin</code> folder, and will not export it to your <code>$PATH</code>.</p>
<br/>
<b>Go to the scripts folder within the monorepo</b>
<pre>
cd scripts
</pre>

<b>Install script</b>

<pre>
./install-websocat.sh
</pre>

<p>
You can see if <code>websocat</code> has been installed by running <code>.bin/websocat</code>.
</p>

  </div>
  </div>
</details>

## Connect to a HOPR node

### 1. Test REST API connectivity

**Accessing our HOPR node REST API documentation**

Your HOPR node comes with [Swagger UI](https://swagger.io/tools/swagger-ui/) documentation showcasing all the exposed API methods available to your
node, and the expected parameters and format to use them.

You can access the UI by visiting `HOPR_NODE_1_HTTP_URL/api/v2/_swagger/#` in your browser.

You can also click here to open [127.0.0.1:3001](http://127.0.0.1:3001/api/v2/_swagger/#) for an individual node or [127.0.0.1:13301](http://127.0.0.1:13301/api/v2/_swagger/#) for the first node in a HOPR cluster.

If your node is running properly, you should see an image similar to this one:

![HOPR network](/img/developer/hopr_swagger_api.png)

**Testing the REST API with `curl`**

Using your node's `apiToken` and your `HOPR_NODE_1_HTTP_URL` from the ["Running a local HOPR Cluster"](/developers/starting-local-cluster) section (likely `127.0.0.1:3001` or `127.0.0.1:13301`), try to send a REST command to query its address with the following `curl`
command. If you don’t have `jq` installed, just remove it from the end of the command.

```bash
echo -n $apiToken | base64 | xargs -I {} curl -s -H "Authorization: Basic {}" $HOPR_NODE_1_HTTP_URL/api/v2/account/address | jq
```

If successful, you should get a response similar to this one:

```json
{
  "nativeAddress": "0x3a54dDE3ee5ACfd43C902cbecC8ED0CBA10Ff326",
  "hoprAddress": "16Uiu2HAmE9b3TSHeF25uJS1Ecf2Js3TutnaSnipdV9otEpxbRN8Q"
}
```

If you've made a mistake, for example forgotting to use `-n` in your `echo` or using the wrong `apiToken`, you’ll see the following instead:

```json
{
  "status": 403,
  "challenge": "Basic realm=hoprd",
  "message": "You must authenticate to access hoprd."
}
```

### 2. Test WebSocket connectivity

Unlike our REST API endpoint, seeing interactions with your HOPR node WebSocket server requires a WebSocket client that will remain open to listen to all messages sent to our HOPR node.

**Connecting to your HOPR node WebSocket server**

With `websocat` installed, please go up one directory: `cd ..` and run the following command to connect to your HOPR node WebSocket server.

```bash
.bin/websocat "$(echo "$HOPR_NODE_1_WS_URL" | sed "s/http/ws/")/?apiToken=$apiToken"
```

:::info Note

Please note that you need to use your `HOPR_NODE_1_WS_URL` (likely `127.0.0.1:3000` or `127.0.0.1:19501`) instead of the `HOPR_NODE_1_HTTP_URL` from the previous step. Your `HOPR_NODE_1_WS_URL` is also referred as `Admin URL` in our tools.

:::

If everything worked correctly, you should see a dump of messages, the last one being:

```json
{ "type": "log", "msg": "ws client connected [ authentication ENABLED ]", "ts": "2022-02-01T19:42:34.152Z" }
```

Now that you are connected, try typing `balance` in the same terminal, which should output as follows:

```json
{"type":"log","msg":"admin > balance\n","ts":"2022-02-01T19:42:35.417Z"}
{"type":"log","msg":"HOPR Balance:  9.6 txHOPR\nETH Balance:   0.99871794476851171 xDAI","ts":"2022-02-01T19:42:35.421Z"}
```

With the connection verified to both our REST and WebSocket endpoints, we can now go ahead and go through the basic functions of the API to send
messages across nodes.

## Additional REST/WebSocket clients

In addition to `curl` and `websocat`, the following clients can also help you connect to your HOPR node fully. Be aware that you will still
need to know your `apiToken`.

**WebSocket clients**

- [Piesocket WebSocket Tester](https://www.piesocket.com/websocket-tester): This is a great tool to debug both listening to and sending
  messages from/to your HOPR node. Make sure to paste your `HOPR_NODE_1_WS_URL` and append your `apiToken` as a query parameter. Also,
  you'll need to change the `http` protocol to `ws`. For instance, here's how this would look in a `Gitpod.io` instance. After it's connected, you can type `balance` to see your node response.

```bash
ws://127.0.0.1:19501/?apiToken=^^LOCAL-testing-123^^
```

If you are using a Gitpod public URL, you can simply use the output of `gp url` for that particular port (`19501`) and paste it in the website.

```bash
gp url 19501
```

The output should look something like this: `wss://19501-hoprnet-mynechat-7x6h2ghc17f.ws-us30.gitpod.io/?apiToken=^^LOCAL-testing-123^^`.

**REST client**

- [ReqBin](https://reqbin.com/): Using their `Custom` header option, you can send the proper `Authorization` request so you can test your
  HOPR node endpoint. For testing, we suggest using `HOPR_NODE_1_HTTP_URL` and the `api/v2/account/address` endpoint. Make sure to use
  the `base64` encoded version of your `apiToken` and add the prefix `Basic `.

:::info Tip
For the standard `apiToken` `^^LOCAL-testing-123^^`, the `base64` encoded value is `Xl5MT0NBTC10ZXN0aW5nLTEyM15e`. To use [ReqBin](https://reqbin.com/)
with a Gitpod exposed URL (e.g. `https://13302-hoprnet-mynechat-7x6h2ghc17f.ws-us30.gitpod.io/api/v2/account/addresses`), you can use `gp url`.
For a different `apiToken` value, you can use the `btoa` function of your browser developer tools to figure it out.

<br/>

**Gitpod command (paste output in the URL section of `reqbin`)**

```bash
gp url 13302
```

**Custom Header**

```bash
Basic Xl5MT0NBTC10ZXN0aW5nLTEyM15e
```

:::

[^1]:
    Although you can successfully interact with a HOPR node only using a WebSocket client, it is recommended to always use the REST API
    to send commands to the HOPR node. This is because the API is optimized for applications, whereas the WebSocket commands are mostly used within the
    `hopr-admin` UI, an operator-targeted tooling used to verify the functionality of the node. In other words, only use the WebSocket server
    when you need to process information sent to a node, and use the REST API when you need to write actions to a node.
