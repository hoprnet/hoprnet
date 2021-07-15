### Testing

Requirements:

Past development has shown that unit testing as well as integration testing can detect basic problems related to the stream usage. Right now, it is known that real Node.JS processes schedule stream I/O operations slightly different than unit tests are executed.

Therefore it is for the moment necessary to test `hopr-connect` with multiple processes that talk through their own network sockets as well as their own instances of WebRTC to each other.

Test setup:

- Charly: a bootstrap server and a relay (first node that will help as a signalling server to connect all subsequent nodes)
- Alice & Bob: 2 clients using Charly as a bootstrap and relay

The clients will use the bootstrap server to determine their own "public" IPv4 addresses. The bootstrap server itself will use external and publicly available bootstrap servers to determine its own public IPv4 address.

```
./scripts/integration-test.sh
```

Alice will attach to a network socket and wait 8 seconds to give Bob time to attach to its network socket. Afterwards, Alice will establish a relayed connection (using Charly as a relay) to Bob and will try to upgrade to a direct WebRTC connection. Both clients are started with a debug flag that prevents from direct connection - even if they were possible.

The test will provide separate logging for all parties, e.g.:

```
Test started
21-07-14T11:45:28Z [hopr-connect-test] alice -> /var/tmp/hopr-connect-alice.log
21-07-14T11:45:28Z [hopr-connect-test] bob -> /var/tmp/hopr-connect-bob.log
21-07-14T11:45:28Z [hopr-connect-test] charly -> /var/tmp/hopr-connect-charly.log
```
