# Decentralised NAT traversal inside HOPR network

Problems:
- many nodes behind NATs, especially receivers of messages
- classical NAT traversal requires central signalling servers which are a single point of failure and are a contradiction to the decentralised-first paradigm
- NAT traversal need to deal with churn, static list of signalling servers per node will not do the job

# Connection establishment

1. Ask `libp2p.getMany(hashed_peerId)` for currently known IP addresses of that node.
2. Send via UDP WebRTC candidates to that node

   This yields a WebRTC negotiated connection
3. Otherwise, ask `libp2p.getProviders(hash(signalling_peerId))` to get all HOPR nodes who are currently working as a signalling server for the desired node.
4. Contact concurrently these nodes as they come in and send them WebRTC candidates and ask them to forward them to the desired node as long as either:

    a. the desired node replies either directly or through the signalling server
    b. WebRTC has detected that a STUN-based connection is not possible, the signalling server will therefore forward all messages between the two of them
    b. the desired node has not replied and there are no other signalling servers at the moment and the request timed out

# Registering as a signalling server

Each node maintains a bounded list of close peers. 'Close' means that the latency to these nodes is particularly small and / or that node is easily accessible them.

Once a node discovers a new peer, they check whether that peer is closer than the previously closest peer. If that is the case, they will ask that node by using `dialProtocol(WEBRTC_BECOME_SIGNALLING_SERVER, { self })` whether it wants to become a signalling server for them.

# Establishing a WebRTC connection

Once a node wants to connect to another node, they will gather some ICE candidates and send them via UDP directly to the desired node. Each candidate has an ID that belongs to a specific connection establishment, such that once they receive ICE candidates they feed them to their corresponding WebRTC instance. Once a WebRTC instance triggers the `connection established` event, they call the connection the connection handler and inform HOPR that there is a new connection.

# Connection lifecycle

Since the number of open connection is restricted due to some network constraints and energy usage / system resource usage considerations, HOPR nodes need to manage their open connections.

Each node maintains a bounded list of open connections. The list is implemented as a heap such that once there is message or packet on one connection, this connection receives an upvote. The upvote leads to an eviction of rarely used connections. Once the maximum number of connections has been reached, they close the connection to those nodes who fall behind the least often used connection.

Summarising votes can be done logarithmically such that counter overflows occur very rarely.

TODO: Combine this with some time degradation such that new connections have a chance to stay open for a while.

This effectively implements a heat map.

# Config:
HOPR IPv4 9091
HOPR IPv6 9092