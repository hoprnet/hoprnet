# Local testing of NAT nodes

This Vagrant configuration create the following setup suitable for isolated local testing of NAT configurations

## Environment

The setup consists of 3 different networks. The first one _192.168.101.0/24_ simulates a public network (e.g Internet). The
other two networks _10.10.0.0/24_ and _172.24.0.0/24_ are simulating local networks behind NAT and can reach the simulated public network.

Vagrant will deploy the following 3 VMs into the 192.168.101.0/24 network. Subsequently, docker-compose is used to
instantiate the HOPR nodes.

- VM (`hoprd-hardhat`, 192.168.101.10)
- VM (`hoprd-public-nodes`, 192.168.101.15)
  - Docker (`hoprd-public-relay`, 192.168.101.17)
- VM (`hoprd-nat-nodes`, 192.168.101.20)
  - Docker (`hoprd-nat-node-1`, 10.10.0.2)
  - Docker (`hoprd-nat-node-2`, 172.24.0.2)

## Node description

### hoprd-hardhat

This VM serves to run Hardhat environment and to allow funding of HOPR nodes.

### hoprd-public-nodes

Used to run a Docker container (`hoprd-public-relay`) that's used to simulate a publicly accessible HOPR node.

### hoprd-nat-nodes

Used to run 2 Docker containers (`hoprd-nat-node-1`, `hoprd-nat-node-2`) that simulate 2 different HOPR nodes behind NATs.

## Shared directories

All 3 VMs map the entire monorepo directory from host to `/opt/hopr`. Changes are synchronized bi-directionally, so any code changes
on the host machine will automatically propagate to the VMs.

The host directory `/tmp/hopr-identities` (created automatically on host) is mapped to `/var/hopr/identities` on all 3 guest VMs. This
directory serves as a storage for identity files that can be used to fund all nodes.

# Installation

- install Vagrant, see https://www.vagrantup.com/docs/installation

On \*nix system, you might want to use `libvirt`, see Vagrant [libvirt plugin](https://github.com/vagrant-libvirt/vagrant-libvirt#installation).

# Usage

Startup your terminal in `scripts/vm-nat-testing`.

## Environment setup

To setup the environment first run:

```shell
vagrant up
```

On \*nix systems:

```shell
vagrant up --provider=libvirt
```

## Basic usage

The public node and both nodes behind NAT are started automatically after provisioning.

On each code change, the following steps are repeatable.

On host machine, you can build HOPR, e.g. using standard:

```shell
yarn ; yarn build
```

Once HOPR is built, public and NAT nodes can be restarted using the following command for the changes to take effect:

```shell
vagrant ssh hoprd-public-nodes -c './startup-hoprd-nodes.sh'
vagrant ssh hoprd-nat-nodes -c './startup-hoprd-nodes.sh'
```

## Funding

Once all nodes have started, each of them created their identity file in `/var/hopr/identities`.
Because the `hoprd-hardhat` VM can also see this directory, we can use it to fund all 3 nodes in one go.

If nodes have not been funded yet, run:

```shell
vagrant ssh hoprd-hardhat -c './faucet.sh'
```

This will fund all the nodes and they should all come up soon.

## Admin panel reachability

- `hoprd-public-relay` can be reached by browser from host on `192.168.101.17:3000`
- `hoprd-nat-node-1` can be reached by browser from host on `192.168.101.20:3010`
- `hoprd-nat-node-2` can be reached by browser from host on `192.168.101.20:3020`

Access token for all 3 is `MyT0ken123^` .

## Tear-down

To stop all the running VMs go to the `scripts/vm-nat-testing` directory and run:

```shell
vagrant halt
```

This will try to gracefully stop the VMs. To restart the environment again, follow the steps in the Environment setup
section again.
Note that whenever the `hoprd-hardhat` VM is stopped, all on-chain information is lost and nodes will need to be funded again.
