### Testing

Requirements:

Past development has shown that unit testing as well as integration testing can detect basic problems related to the stream usage. Right now, it is known that real Node.JS processes schedule stream I/O operations slightly different than unit tests are executed.

Therefore it is for the moment necessary to test `hopr-connect` with multiple processes that talk through their own network sockets as well as their own instances of WebRTC to each other.

Test setup:

- bootstrap server (first node that will help as a signalling server to connect all subsequent nodes)
- 2 or more clients

The clients will use the bootstrap server to determine their own "public" IPv4 addresses. The bootstrap server itself will use external and publicly available bootstrap servers to determine its own public IPv4 address.

```sh
# server
DEBUG=hopr-connect*,simple-peer ts-node testing/server.ts

# clients
DEBUG=hopr-connect*,simple-peer ts-node testing/client.ts 1
DEBUG=hopr-connect*,simple-peer ts-node testing/client.ts 0
```

Client 0 will attach to a network socket and wait ~ 15 seconds to give client 1 time to attach to its network socket. Afterwards, client 0 will establish a relayed connection to client 1 and will try to upgrade to a direct WebRTC connection. Both clients are started with a debug flag that prevents from direct connection - even if they were possible.
