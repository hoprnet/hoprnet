## Network Registry

The network registry is used to control nodes' access to the network.
In the monte_rosa network, the network registry maintains the link between staking accounts and HOPR node peer IDs.
Starting from the provindence release, network registry peer IDs are deprecated and replaced by HOPR node Ethereum addresses.
This design document specifically describes the network registry for the provindence release onwards.

### Architecture

(See [diagram](https://whimsical.com/network-registry-RcSfnsAxeJ5tMcCZP4xHXQ))

### Functionalities in Network Registry

In the local development environment, the design is simplified to allow all nodes to join the local network.

In the production environment:

- The network registry is a singleton that can be used for multiple networks.
- The network registry can be globally enabled/disabled. A disabled network registry allows arbitrary nodes to join the network.
- The network registry communicates with a "network registry proxy," where the logic for checking if a staking account is allowed to register HOPR nodes is stored.
- The network registry can be managed by multiple manager accounts.
- Eligible staking accounts can register nodes by themselves. The maximum number of nodes is limited based on the results returned by the network registry proxy.
- A registered node can be removed from the network registry by its staking account.
- Managers can register nodes for staking accounts if the nodes haven't been registered under any staking account.
- Managers can deregister nodes for staking accounts if the nodes are registered.
- In case a manager changes the staking account associated with a node, the manager should first deregister the node and then register it again.
- As the check for how many nodes a staking account can register is asynchronous, meaning that changes to the criteria won't affect registered nodes, there is a "sync" functionality that checks the staking account based on the latest requirement.
- The sync function can be called by a staking account for themselves.
- Managers can call the sync function for multiple staking accounts.
- In local development environment, the design is simplified to allow all the nodes joining the local network.

### Functionalities in Network Registry Proxy

There are three "network registry proxies" (NR proxies) used in HOPR network:

- HoprDummyProxyForNetworkRegistry
- HoprStakingProxyForNetworkRegistry
- HoprSafeProxyForNetworkRegistry

This section specifically describes the third "NR proxy for Safe" as it will be used in the upcoming networks.

This "NR proxy for Safe":

- Is controlled by one or multiple manager accounts.
- When staking accounts add/remove nodes to/from the network registry, the maximum number of nodes a staking account is allowed to register is calculated as the floor value of the division between the "wxHOPR balance of a Hopr Safe" and the "staking threshold."
- If a manager wants to disable the ability for staking accounts to register nodes by themselves (and only allow managers to include/remove nodes), the manager should call updateStakeThreshold(0). By setting the threshold to zero, the "maxAllowedRegistrations" per node becomes not-a-number.
- Managers can also update the snapshot block number so that Safe accounts will return the balance at the new snapshot block.
