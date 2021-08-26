<!-- INTRODUCTION -->
<p align="center">
  <a href="https://hoprnet.org" target="_blank" rel="noopener noreferrer">
    <img width="100" src="https://github.com/hoprnet/hopr-assets/blob/master/v1/logo/hopr_logo_padded.png?raw=true" alt="HOPR Logo">
  </a>
  
  <!-- Title Placeholder -->
  <h3 align="center">HOPR</h3>
  <p align="center">
    <code>A project by the HOPR Association</code>
  </p>
  <p align="center">
    HOPR is a privacy-preserving messaging protocol which enables the creation of a secure communication network via relay nodes powered by economic incentives using digital tokens.
  </p>
  <p align="center">
    <img src="https://img.shields.io/badge/Gitpod-ready--to--code-blue?logo=gitpod" alt="Gitpod">
  </p>
</p>

## Table of Contents

- [Table of Contents](#table-of-contents)
- [Getting Started](#getting-started)
- [Install](#install)
  - [Install via NPM](#install-via-npm)
  - [Install via Docker](#install-via-docker)
  - [Install via Nix package manager](#install-via-nix-package-manager)
- [Using](#using)
  - [Using NPM](#using-npm)
  - [Using Docker](#using-docker)
- [Develop](#develop)
- [Test](#test)
  - [Github Actions CI](#github-actions-ci)
  - [End-to-End Testing](#end-to-end-testing)
    - [Running Tests Locally](#running-tests-locally)
    - [Running Tests on Google Cloud Platform](#running-tests-on-google-cloud-platform)
- [Deploy](#deploy)
  - [Using Google Cloud Platform](#using-google-cloud-platform)
  - [Using Google Cloud Platform and a Default Topology](#using-google-cloud-platform-and-a-default-topology)
- [Tooling](#tooling)
- [Contact](#contact)
- [License](#license)

## Getting Started

A good place to start is the
[Getting Started guide on YouTube][7] which walks through the following
instructions using GitPod.

## Install

The following instructions show how the latest community release may be
installed. The instructions should be adapted if you want to use the latest
development release or any other older release.

### Install via NPM

Using the [hoprd npm package][6]:

```sh
mkdir MY_NEW_HOPR_TEST_FOLDER
cd MY_NEW_HOPR_TEST_FOLDER
npm install @hoprnet/hoprd@1.73
```

### Install via Docker

All our docker images can be found in [our Google Cloud Container Registry][4].
Each image is prefixed with `gcr.io/hoprassociation/$PROJECT:$RELEASE`.
The `latest` tag represents the `master` branch, while the `latest-constantine` tag
represents the most recent `release/*` branch.

You can pull the Docker image like so:

```sh
docker pull gcr.io/hoprassociation/hoprd:latest-constantine
```

For ease of use you can set up a shell alias to run the latest release as a docker container:

```sh
alias hoprd='docker run --pull always -ti -v ${HOPRD_DATA_DIR:-$HOME/.hoprd-db}:/app/db -p 9091:9091 -p 3000:3000 -p 3001:3001 gcr.io/hoprassociation/hoprd:latest-constantine'
```

**IMPORTANT:** Using the above command will map the database folder used by hoprd to a local folder called `.hoprd-db` in your home directory. You can customize the location of that folder further by executing the following command:

```sh
HOPRD_DATA_DIR=${HOME}/.hoprd-better-db-folder eval hoprd
```

Also all ports are mapped to your local host, assuming you stick to the default port numbers.

### Install via [Nix package manager][1]

NOTE: This setup should only be used for development or if you know what you
are doing and don't neetd further supported. Otherwise you should use the `npm`
or `docker` setup.

You will need to clone the `hoprnet` repo first:

```sh
git clone https://github.com/hoprnet/hoprnet
```

If you have [direnv][2] set up properly your `nix-shell` will be
configured automatically upon entering the `hoprnet` directory and enabling it
via `direnv allow`. Otherwise you must enter the `nix-shell` manually:

```sh
nix develop
```

Now you may follow the instructions in [Develop](#develop).

## Using

The `hoprd` provides various command-line switches to configure its behaviour. For reference these are documented here as well:

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
  --adminPort                   Port to listen to for admin console  [default: 3000]
  --testAnnounceLocalAddresses  For testing local testnets. Announce local addresses.  [boolean] [default: false]
  --testPreferLocalAddresses    For testing local testnets. Prefer local peers to remote.  [boolean] [default: false]
  --testUseWeakCrypto           weaker crypto for faster node startup  [boolean] [default: false]
  --testNoAuthentication        (experimental) disable remote authentication
```

As you might have noticed running the node without any command-line argument might not work depending on the installation method used. Here are examples to run a node with some safe configurations set.

### Using NPM

The following command assumes you've setup a local installation like described in [Install via NPM](#install-via-npm).

```sh
cd MY_NEW_HOPR_TEST_FOLDER
DEBUG=hopr* npx hoprd --admin --init --announce --identity .hopr-identity --password switzerland --forwardLogs --apiToken <MY_TOKEN>
```

Here is a short break-down of each argument.

```sh
hoprd
  --admin   	                         # enable the node's admin UI, available at localhost:3000
  --init 				 # initialize the database and identity if not present
  --announce 				 # announce the node to other nodes in the network and act as relay if publicly reachable
  --identity .hopr-identity              # store your node identity information in your test folder
  --password switzerland   		 # set the encryption password for your identity
  --forwardLogs                          # enable the node's log forwarding to the ceramic network
  --apiToken <MY_TOKEN> # specify password for accessing admin panel and REST API (REQUIRED)
```

### Using Docker

The following command assumes you've setup an alias like described in [Install via Docker](#install-via-docker).

```sh
hoprd --identity /app/db/.hopr-identity --password switzerland --init --announce --host "0.0.0.0:9091" --admin --adminHost 0.0.0.0 --forwardLogs
```

Here is a short break-down of each argument.

```sh
hoprd
  --identity /app/db/.hopr-identity      # store your node identity information in the persisted database folder
  --password switzerland   		 # set the encryption password for your identity
  --init 				 # initialize the database and identity if not present
  --announce 				 # announce the node to other nodes in the network and act as relay if publicly reachable
  --host "0.0.0.0:9091"   		 # set IP and port of the P2P API to the container's external IP so it can be reached on your host
  --admin   	                         # enable the node's admin UI
  --adminHost 0.0.0.0                    # set IP of the Rest API to the container's external IP so it can be reached on your host
  --forwardLogs                          # enable the node's log forwarding to the ceramic network
  --apiToken <MY_TOKEN> # specify password for accessing admin panel and REST API(REQUIRED)
```

## Develop

```sh
yarn          # Install lerna and sets project up
yarn build    # Builds contracts, clients, etc

# starting network
yarn run:network

# running normal node alice (separate terminal)
DEBUG=hopr* yarn run:hoprd:alice

# running normal node bob (separate terminal)
DEBUG=hopr* yarn run:hoprd:bob

# fund all your nodes to get started
yarn run:faucet:all
```

## Test

### Unit testing

We use [mocha][9] for our tests. You can run our test suite across all
packages using the following command:

```sh
yarn test
```

To run tests of a single package (e.g. hoprd) execute:

```sh
yarn --cwd packages/hoprd test
```

To run tests of a single test suite (e.g. Identity) within a
package (e.g. hoprd) execute:

For instance, to run only the `Identity` test suite in `hoprd`, you need to
run the following:

```sh
yarn --cwd packages/hoprd test --grep "Identity"
```

In a similar fashion, our contracts can be tested in isolation. For now, you
need to pass the file to be tested, as [hardhat does not support --grep][12]

```sh
yarn test:contracts test/HoprChannels.spec.ts
```

In case a package you need to test is not included in our `package.json`,
please feel free to update it as needed.

#### Test-driven development

To make sure we add the least amount of untested code to our codebase,
whenever possible all code should come accompanied by a test. To do so,
locate the `.spec` or equivalent test file for your code. If it does not
exist, create it within the same file your code will live in.

Afterwards, ensure you create a breaking test for your feature. For example,
the [following commit][10] added a test to a non-existing feature. The
immediate [commit][11] provided the actual feature for that given test. Repeat
this process for all the code you add to our codebase.

_(The code was pushed as an example, but ideally, you only push code that has
working tests on your machine, as to avoid overusing our CI pipeline with
known broken tests.)_

### Github Actions CI

We run a fair amount of automation using Github Actions. To ease development
of these workflows one can use [act][8] to run workflows locally in a
Docker environment.

E.g. running the build workflow:

```sh
act -j build
```

For more information please refer to [act][8]'s documentation.

### End-to-End Testing

#### Running Tests Locally

End-to-end testing is usually performed by the CI, but can also be performed
locally by executing:

```sh
./scripts/run-integration-tests-source.sh
```

Read the full help information of the script in case of questions:

```sh
./scripts/run-integration-tests-source.sh --help
```

That command will spawn multiple `hoprd` nodes locally from the local
source-code and run the tests against this cluster of nodes. The tests can be
found in the files `test/*.sh`. The script will cleanup all nodes once completed
unless instructed otherwise.

An alternative to using the local source-code is running the tests against
a NPM package.

```sh
./scripts/run-integration-tests-npm.sh
```

If no parameter is given the NPM package which correlates to the most recent Git
tag will be used, otherwise the first parameter is used as the NPM package
version to test.

Read the full help information of the script in case of questions:

```sh
./scripts/run-integration-tests-npm.sh --help
```

#### Running Tests on Google Cloud Platform

In some unique cases, some bugs might not had been picked up by our end-to-end
testing and instead only show up when deployed to production. To avoid having
to see these only after a time consuming build, a cluster of nodes can be
deployed to Google Cloud Platform which is then used to run tests against it.

A requirement for this setup is a working `gcloud` configuration locally.
The easiest approach would be to authenticate with `gcloud auth login`.

The cluster creation and tests can be run with:

```sh
FUNDING_PRIV_KEY=mysecretaccountprivkey \
  ./scripts/run-integration-tests-gcloud.sh
```

The given account private key is used to fund the test nodes to be able to
perform throughout the tests. Thus the account must have enough funds available.

Read the full help information of the script in case of questions:

```sh
./scripts/run-integration-tests-gcloud.sh --help
```

## Deploy

The deployment nodes and networks is mostly orchestrated through the script
files in `scripts/` which are executed by the Github Actions CI workflows.
Therefore, all common and minimal networks do not require manual steps to be
deployed.

### Using Google Cloud Platform

However, sometimes it is useful to deploy additional nodes or specific versions
of `hoprd`. To accomplish that its possible to create a cluster on GCP using the
following scripts:

```sh
./scripts/setup-gcloud-cluster.sh my-custom-cluster-without-name
```

Read the full help information of the script in case of questions:

```sh
./scripts/setup-gcloud-cluster.sh --help
```

The script requires a few environment variables to be set, but will inform the
user if one is missing. It will create a cluster of 6 nodes. By default these
nodes will use the latest Docker image of `hoprd` and run on the `Goerli`
network. Different versions and different target networks can be configured
through the parameters and environment variables.

To launch nodes using the `xDai` network one would execute (with the
placeholders replaced accordingly):

```sh
HOPRD_PROVIDER="<URL_TO_AN_XDAI_ENDPOINT>" \
HOPRD_TOKEN_CONTRACT="<ADDRESS_OF_TOKEN_CONTRACT_ON_XDAI>" \
  ./scripts/setup-gcloud-cluster.sh my-custom-cluster-without-name
```

A previously started cluster can be destroyed, which includes all running nodes,
by using the same script but setting the cleanup switch:

```sh
HOPRD_PERFORM_CLEANUP=true \
  ./scripts/setup-gcloud-cluster.sh my-custom-cluster-without-name
```

### Using Google Cloud Platform and a Default Topology

The creation of a `hoprd` cluster on GCP can be enhanced by providing a topology
script to the creation script:

```sh
./scripts/setup-gcloud-cluster.sh \
  my-custom-cluster-without-name \
  gcr.io/hoprassociation/hoprd:latest \
  `pwd`/scripts/topologies/full_interconnected_cluster.sh
```

After the normal cluster creation the topology script will then open channels
between all nodes so they are fully interconnected. Custom topology scripts can
be easily added and used in the same manner. Refer to the referenced scripts as
a guideline on how to get started.

## Tooling

As some tools are only partially supported, please tag the respective team member
whenever you need an issue about a particular tool.

| Maintainer       | Technology  |
| :--------------- | :---------: |
| @jjperezaguinaga | Visual Code |
| @tolbrino        |     Nix     |

## Contact

- [Twitter](https://twitter.com/hoprnet)
- [Telegram](https://t.me/hoprnet)
- [Medium](https://medium.com/hoprnet)
- [Reddit](https://www.reddit.com/r/HOPR/)
- [Email](mailto:contact@hoprnet.org)
- [Discord](https://discord.gg/5FWSfq7)
- [Youtube](https://www.youtube.com/channel/UC2DzUtC90LXdW7TfT3igasA)

## License

[GPL v3](LICENSE) © HOPR Association

[1]: https://nixos.org/learn.html
[2]: https://search.nixos.org/packages?channel=20.09&show=direnv&from=0&size=50&sort=relevance&query=direnv
[4]: https://console.cloud.google.com/gcr/images/hoprassociation/GLOBAL
[6]: https://www.npmjs.com/package/@hoprnet/hoprd
[7]: https://www.youtube.com/watch?v=d0Eb6haIUu4
[8]: https://github.com/nektos/act
[9]: https://mochajs.org/
[10]: https://github.com/hoprnet/hoprnet/pull/1974/commits/331d6e99d1199250a302211be7b8dd9a22fa6e23#diff-83e70acfe04a8f13821ff96a1115f02a4b683a6370568ba9beea16da6d0c2cffR33-R49
[11]: https://github.com/hoprnet/hoprnet/pull/1974/commits/53663517309d0f8918c5066fd98503afe8d8dd76#diff-9bf7c02325c8f5b6330a15a745a3ad736ee139a78c28a15d594756c406378884R91-R96
[12]: https://github.com/nomiclabs/hardhat/issues/1116
