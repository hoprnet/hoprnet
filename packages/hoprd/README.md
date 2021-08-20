# HOPR Admin

Runs a HOPR Node and the HOPR Admin interface.

## Usage

```
hoprd [OPTION]...
```

### Options

See `hoprd --help` for full list.

```sh
$ hoprd --help
Options:
  --help                        Show help  [boolean]
  --version                     Show version number  [boolean]
  --network                     Which network to run the HOPR node on  [choices: "ETHEREUM"] [default: "ETHEREUM"]
  --provider                    A provider url for the Network you specified  [default: "https://still-patient-forest.xdai.quiknode.pro/f0cdbd6455c0b3aea8512fc9e7d161c1c0abf66a/"]
  --host                        The network host to run the HOPR node on.  [default: "0.0.0.0:9091"]
  --announce                    Announce public IP to the network  [boolean] [default: false]
  --admin                       Run an admin interface on localhost:3000, requires --apiToken  [boolean] [default: false]
  --rest                        Run a rest interface on localhost:3001, requires --apiToken  [boolean] [default: false]
  --restHost                    Updates the host for the rest server  [default: "localhost"]
  --restPort                    Updates the port for the rest server  [default: 3001]
  --healthCheck                 Run a health check end point on localhost:8080  [boolean] [default: false]
  --healthCheckHost             Updates the host for the healthcheck server  [default: "localhost"]
  --healthCheckPort             Updates the port for the healthcheck server  [default: 8080]
  --forwardLogs                 Forwards all your node logs to a public available sink  [boolean] [default: false]
  --forwardLogsProvider         A provider url for the logging sink node to use  [default: "https://ceramic-clay.3boxlabs.com"]
  --password                    A password to encrypt your keys  [default: ""]
  --apiToken                    (experimental) A REST API token and admin panel password for user authentication  [string]
  --identity                    The path to the identity file  [default: "/home/tbr/.hopr-identity"]
  --run                         Run a single hopr command, same syntax as in hopr-admin  [default: ""]
  --dryRun                      List all the options used to run the HOPR node, but quit instead of starting  [boolean] [default: false]
  --data                        manually specify the database directory to use  [default: ""]
  --init                        initialize a database if it doesn't already exist  [boolean] [default: false]
  --privateKey                  A private key to be used for your node wallet, to quickly boot your node [string] [default: undefined]
  --adminHost                   Host to listen to for admin console  [default: "localhost"]
  --adminPort                   Port to listen to for admin console  [default: 3000
  --environment                 Environment id to run in [string] [default: defined by release] 
  --testAnnounceLocalAddresses  For testing local testnets. Announce local addresses.  [boolean] [default: false]
  --testPreferLocalAddresses    For testing local testnets. Prefer local peers to remote.  [boolean] [default: false]
  --testUseWeakCrypto           weaker crypto for faster node startup  [boolean] [default: false]
  --testNoAuthentication        (experimental) disable remote authentication
```
