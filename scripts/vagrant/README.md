# Local testing of NAT nodes

This Vagrant configuration create the following setup suitable for isolated local testing of NAT configurations

## Environment

The setup consists of 3 different networks. The first one _192.168.101.0/24_ simulates a public network (e.g Internet). The
other two networks _10.10.0.0/24_ and _172.24.0.0/24_ are simulating local networks behind NAT and can reach the simulated public network.

Vagrant will deploy the following 3 VMs into the 192.168.101.0/24 network. Subsequently, docker-compose is used to
instantiate the HOPR nodes.

- VM (`hardhat`, 192.168.101.10)
- VM (`public`, 192.168.101.15)
  - Docker (`hoprd-public-relay`, 192.168.101.17)
- VM (`nat-nodes`, 192.168.101.20)
  - Docker (`hoprd-nat-node-1`, 10.10.0.2)
  - Docker (`hoprd-nat-node-2`, 172.24.0.2)

## Node description

### hardhat

This VM serves to run Hardhat environment and to allow funding of HOPR nodes.

### public

Used to run a Docker container (`hoprd-public-relay`) that's used to simulate a publicly accessible HOPR node.

### nat-nodes

Used to run 2 Docker containers (`hoprd-nat-node-1`, `hoprd-nat-node-2`) that simulate 2 different HOPR nodes behind NATs.

## Shared directories

All 3 VMs map the entire monorepo directory from host to `/opt/hopr`. Changes are synchronized bi-directionally, so any code changes
on the host machine will automatically propagate to the VMs.

The host directory `/tmp/hopr-identities` (created automatically on host) is mapped to `/var/hopr/identities` on all 3 guest VMs. This
directory serves as a storage for identity files that can be used to fund all nodes.

# Usage

It is recommended to run 4 different terminal windows **A**,**B**,**C** (with `scripts/vagrant` current directory) each of which will talk to one of the 3 VMs
and **D** which will be your local terminal with current directory in the base of the monorepo.

## Environment setup

To setup the environment first run:

```shell
vagrant up
```

Now lets connect the terminals to the respective VM.

In **terminal A** run:

```shell
vagrant ssh hardhat
```

In **terminal B** run:

```shell
vagrant ssh public
```

In **terminal C** run:

```shell
vagrant ssh nat-nodes
```

In **terminal A** startup the Hardhat network and wait for the script to finish.

```shell
./startup-network.sh
```

This finalizes the environment setup.

## Basic usage

The following steps are repeatable.

In **terminal D** you can build HOPR, e.g. using standard:

```shell
yarn ; yarn build
```

Once HOPR is built, let's move to a **terminal window B** and start the public HOPR node `hoprd-public-relay` by running:

```shell
./startup-public.sh
```

Let's move to the **terminal window C** and start the HOPR nodes behind NAT (`hoprd-nat-node-1`, `hoprd-nat-node-2`)

```shell
./startup-nat-nodes.sh
```

If code changes in the monorepo, you can use `Ctrl+C` to quit the running HOPR nodes in **terminal B** and **terminal C**,
and re-run the above 3 steps to build & start nodes again.

## Funding

Once all nodes have started, each of them created their identity file in `/var/hopr/identities`.
Because the `hardhat` VM can also see this directory, we can use it to fund all 3 nodes in one go.

If nodes have not been funded yet, go to **terminal window A** and run:

```shell
./faucet.sh
```

This will fund all the nodes and they should all come up soon.

After nodes have been funded, the **terminal window A** is not needed anymore and can be closed.
This leaves you with terminal windows **B**, **C** and **D**.

## Tear-down

To stop all the running VMs go to the `scripts/vagrant` directory and run:

```shell
vagrant halt
```

This will try to gracefully stop the VMs. To restart the environment again, follow the steps in the Environment setup
section again.
