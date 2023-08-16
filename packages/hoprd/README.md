# HOPR Admin

Runs a HOPR Node and the HOPR Admin interface.

When the Rest API is enabled, the node serves a Swagger UI to inspect and test
the Rest API v3 at:

http://localhost:3001/api/v3/\_swagger

NOTE: Hostname and port can be different, since they depend on the settings `--apiHost` and `--apiPort`.

## Usage

```
hoprd [OPTION]...
```

### Options

See `hoprd --help` for full list.

```sh
$ hoprd --help
Options:
      --network <NETWORK>
          Network id which the node shall run on [env: HOPRD_NETWORK=] [possible values: anvil-localhost, rotsee, debug-staging, anvil-localhost2, monte_rosa]
      --identity <identity>
          The path to the identity file [env: HOPRD_IDENTITY=] [default: /home/tino/.hopr-identity]
      --data <data>
          manually specify the data directory to use [env: HOPRD_DATA=] [default: /home/tino/work/hopr/hoprnet/packages/hoprd/hoprd-db]
      --host <HOST>
          Host to listen on for P2P connections [env: HOPRD_HOST=] [default: 0.0.0.0:9091]
      --announce
          Run as a Public Relay Node (PRN) [env: HOPRD_ANNOUNCE=]
      --api
          Expose the API on localhost:3001 [env: HOPRD_API=]
      --apiHost <HOST>
          Set host IP to which the API server will bind [env: HOPRD_API_HOST=] [default: localhost]
      --apiPort <PORT>
          Set port to which the API server will bind [env: HOPRD_API_PORT=] [default: 3001]
      --apiToken <TOKEN>
          A REST API token and for user authentication [env: HOPRD_API_TOKEN=]
      --healthCheck
          Run a health check end point on localhost:8080 [env: HOPRD_HEALTH_CHECK=]
      --healthCheckHost <HOST>
          Updates the host for the healthcheck server [env: HOPRD_HEALTH_CHECK_HOST=] [default: localhost]
      --healthCheckPort <PORT>
          Updates the port for the healthcheck server [env: HOPRD_HEALTH_CHECK_PORT=] [default: 8080]
      --password <PASSWORD>
          A password to encrypt your keys [env: HOPRD_PASSWORD=]
      --provider <PROVIDER>
          A custom RPC provider to be used for the node to connect to blockchain [env: HOPRD_PROVIDER=]
      --dryRun
          List all the options used to run the HOPR node, but quit instead of starting [env: HOPRD_DRY_RUN=]
      --init
          initialize a database if it doesn't already exist [env: HOPRD_INIT=]
      --allowLocalNodeConnections
          Allow connections to other nodes running on localhost [env: HOPRD_ALLOW_LOCAL_NODE_CONNECTIONS=]
      --allowPrivateNodeConnections
          Allow connections to other nodes running on private addresses [env: HOPRD_ALLOW_PRIVATE_NODE_CONNECTIONS=]
      --testAnnounceLocalAddresses
          For testing local testnets. Announce local addresses [env: HOPRD_TEST_ANNOUNCE_LOCAL_ADDRESSES=]
      --heartbeatInterval <MILLISECONDS>
          Interval in milliseconds in which the availability of other nodes get measured [env: HOPRD_HEARTBEAT_INTERVAL=] [default: 60000]
      --heartbeatThreshold <MILLISECONDS>
          Timeframe in milliseconds after which a heartbeat to another peer is performed, if it hasn't been seen since [env: HOPRD_HEARTBEAT_THRESHOLD=] [default: 60000]
      --heartbeatVariance <MILLISECONDS>
          Upper bound for variance applied to heartbeat interval in milliseconds [env: HOPRD_HEARTBEAT_VARIANCE=] [default: 2000]
      --onChainConfirmations <CONFIRMATIONS>
          Number of confirmations required for on-chain transactions [env: HOPRD_ON_CHAIN_CONFIRMATIONS=] [default: 8]
      --networkQualityThreshold <THRESHOLD>
          Miniumum quality of a peer connection to be considered usable [env: HOPRD_NETWORK_QUALITY_THRESHOLD=] [default: 0.5]
  -h, --help
          Print help
  -V, --version
          Print version

All CLI options can be configured through environment variables as well. CLI parameters have precedence over environment variables.
```
