@hoprnet/hoprd / [Exports](modules.md)

# HOPR Admin

Runs a HOPR Node and the HOPR Admin interface.

When the Rest API is enabled, the node serves a Swagger UI to inspect and test
the Rest API v2 at:

http://localhost:3001/api/v2/\_swagger

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
  --help                         Show help  [boolean]
  --version                      Show version number  [boolean]
  --environment                  Environment id which the node shall run on (HOPRD_ENVIRONMENT)  [string] [choices: "anvil-localhost", "anvil-localhost2", "master-staging", "debug-staging", "tuttlingen", "prague", "budapest", "athens", "lisbon", "ouagadougou", "paleochora", "monte_rosa"] [default: ""]
  --host                         The network host to run the HOPR node on [env: HOPRD_HOST]  [string] [default: "0.0.0.0:9091"]
  --announce                     Announce public IP to the network [env: HOPRD_ANNOUNCE]  [boolean] [default: false]
  --api                          Expose the API on localhost:3001, requires --apiToken. [env: HOPRD_API]  [boolean] [default: false]
  --apiHost                      Set host IP to which the API server will bind. [env: HOPRD_API_HOST]  [string] [default: "localhost"]
  --apiPort                      Set host port to which the API server will bind. [env: HOPRD_API_PORT]  [number] [default: 3001]
  --healthCheck                  Run a health check end point on localhost:8080 [env: HOPRD_HEALTH_CHECK]  [boolean] [default: false]
  --healthCheckHost              Updates the host for the healthcheck server [env: HOPRD_HEALTH_CHECK_HOST]  [string] [default: "localhost"]
  --healthCheckPort              Updates the port for the healthcheck server [env: HOPRD_HEALTH_CHECK_PORT]  [number] [default: 8080]
  --password                     A password to encrypt your keys [env: HOPRD_PASSWORD]  [string] [default: ""]
  --apiToken                     A REST API token for user authentication [env: HOPRD_API_TOKEN]  [string]
  --privateKey                   A private key to be used for the node [env: HOPRD_PRIVATE_KEY]  [string]
  --provider                     A custom RPC provider to be used for the node to connect to blockchain [env: HOPRD_PROVIDER]  [string]
  --identity                     The path to the identity file [env: HOPRD_IDENTITY]  [string] [default: "/home/tino/.hopr-identity"]
  --dryRun                       List all the options used to run the HOPR node, but quit instead of starting [env: HOPRD_DRY_RUN]  [boolean] [default: false]
  --data                         manually specify the data directory to use [env: HOPRD_DATA]  [string] [default: "/home/tino/work/hopr/hoprnet/packages/hoprd"]
  --init                         initialize a database if it doesn't already exist [env: HOPRD_INIT]  [boolean] [default: false]
  --allowLocalNodeConnections    Allow connections to other nodes running on localhost [env: HOPRD_ALLOW_LOCAL_NODE_CONNECTIONS]  [boolean] [default: false]
  --allowPrivateNodeConnections  Allow connections to other nodes running on private addresses [env: HOPRD_ALLOW_PRIVATE_NODE_CONNECTIONS]  [boolean] [default: false]
  --testAnnounceLocalAddresses   For testing local testnets. Announce local addresses [env: HOPRD_TEST_ANNOUNCE_LOCAL_ADDRESSES]  [boolean] [default: false]
  --testPreferLocalAddresses     For testing local testnets. Prefer local peers to remote [env: HOPRD_TEST_PREFER_LOCAL_ADDRESSES]  [boolean] [default: false]
  --testUseWeakCrypto            weaker crypto for faster node startup [env: HOPRD_TEST_USE_WEAK_CRYPTO]  [boolean] [default: false]
  --disableApiAuthentication     Disable authentication for the API endpoints [env: HOPRD_DISABLE_API_AUTHENTICATION]  [boolean] [default: false]
  --heartbeatInterval            Interval in milliseconds in which the availability of other nodes get measured [env: HOPRD_HEARTBEAT_INTERVAL]  [number] [default: 60000]
  --heartbeatThreshold           Timeframe in milliseconds after which a heartbeat to another peer is performed, if it hasn't been seen since [env: HOPRD_HEARTBEAT_THRESHOLD]  [number] [default: 60000]
  --heartbeatVariance            Upper bound for variance applied to heartbeat interval in milliseconds [env: HOPRD_HEARTBEAT_VARIANCE]  [number] [default: 2000]
  --networkQualityThreshold      Miniumum quality of a peer connection to be considered usable [env: HOPRD_NETWORK_QUALITY_THRESHOLD]  [number] [default: 0.5]
  --onChainConfirmations         Number of confirmations required for on-chain transactions [env: HOPRD_ON_CHAIN_CONFIRMATIONS]  [number] [default: 8]

All CLI options can be configured through environment variables as well. CLI parameters have precedence over environment variables.
```
