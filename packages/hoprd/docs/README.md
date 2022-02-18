@hoprnet/hoprd / [Exports](modules.md)

# HOPR Admin

Runs a HOPR Node and the HOPR Admin interface.

When the Rest API is enabled, the node serves a Swagger UI to inspect and test
the Rest API v2 at:

http://localhost:3001/api/v2/\_swagger

NOTE: Hostname and port can be different, since they depend on the settings `--restHost` and `--restPort`.

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
  --environment                  Environment id which the node shall run on  [string] [choices: "hardhat-localhost", "hardhat-localhost2", "master-goerli", "debug-goerli", "tuttlingen", "prague", "budapest", "athens", "lisbon"] [default: ""]
  --host                         The network host to run the HOPR node on.  [string] [default: "0.0.0.0:9091"]
  --announce                     Announce public IP to the network  [boolean] [default: false]
  --admin                        Run an admin interface on localhost:3000, requires --apiToken  [boolean] [default: false]
  --adminHost                    Host to listen to for admin console  [string] [default: "localhost"]
  --adminPort                    Port to listen to for admin console  [string] [default: 3000]
  --api, --rest                  Expose the Rest (V1, V2) and Websocket (V2) API on localhost:3001, requires --apiToken. "--rest" is deprecated.  [boolean] [default: false]
  --apiHost, --restHost          Set host IP to which the Rest and Websocket API server will bind. "--restHost" is deprecated.  [string] [default: "localhost"]
  --apiPort, --restPort          Set host port to which the Rest and Websocket API server will bind. "--restPort" is deprecated.  [number] [default: 3001]
  --healthCheck                  Run a health check end point on localhost:8080  [boolean] [default: false]
  --healthCheckHost              Updates the host for the healthcheck server  [string] [default: "localhost"]
  --healthCheckPort              Updates the port for the healthcheck server  [number] [default: 8080]
  --forwardLogs                  Forwards all your node logs to a public available sink  [boolean] [default: false]
  --forwardLogsProvider          A provider url for the logging sink node to use  [string] [default: "https://ceramic-clay.3boxlabs.com"]
  --password                     A password to encrypt your keys  [string] [default: ""]
  --apiToken                     A REST API token and admin panel password for user authentication  [string]
  --privateKey                   A private key to be used for your HOPR node  [string]
  --identity                     The path to the identity file  [string] [default: "/home/tino/.hopr-identity"]
  --run                          Run a single hopr command, same syntax as in hopr-admin  [string] [default: ""]
  --dryRun                       List all the options used to run the HOPR node, but quit instead of starting  [boolean] [default: false]
  --data                         manually specify the database directory to use  [string] [default: ""]
  --init                         initialize a database if it doesn't already exist  [boolean] [default: false]
  --allowLocalNodeConnections    Allow connections to other nodes running on localhost.  [boolean] [default: false]
  --allowPrivateNodeConnections  Allow connections to other nodes running on private addresses.  [boolean] [default: false]
  --testAnnounceLocalAddresses   For testing local testnets. Announce local addresses.  [boolean] [default: false]
  --testPreferLocalAddresses     For testing local testnets. Prefer local peers to remote.  [boolean] [default: false]
  --testUseWeakCrypto            weaker crypto for faster node startup  [boolean] [default: false]
  --testNoAuthentication         no remote authentication for easier testing  [boolean] [default: false]
```
