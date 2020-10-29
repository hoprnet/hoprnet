# HOPR Admin

Runs a HOPR Node and the HOPR Admin interface.

## Usage

```
hoprd [OPTION]...
```

### Options

See `hoprd help` for full list.

```
  --help      Show help                                                                                        [boolean]
  --version   Show version number                                                                              [boolean]
  --network   Which network to run the HOPR node on                          [choices: "ETHEREUM"] [default: "ETHEREUM"]
  --provider  A provider url for the Network you specified
                                               [default: "wss://kovan.infura.io/ws/v3/f7240372c1b442a6885ce9bb825ebc36"]
  --host      The network host to run the HOPR node on.                                        [default: "0.0.0.0:9091"]
  --admin     Run an admin interface on localhost:3000                                        [boolean] [default: false]
  --password  A password to encrypt your keys                                                              [default: ""]
  --run       Run a single hopr command, same syntax as in hopr-admin                                     [default: ""]

```
