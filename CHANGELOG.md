New:
- Listening on IPv6 sockets is supported 
- Use WebRTC by default and let WebRTC decide which transport protocol will be used
- `yarn demo` spawns its own mini-testnet, including bootstrap server and persistent blockchain

Changed:
- crawling: crawling is not block anymore, leads to faster crawling
- heartbeat: every connection uses its own timer now

Fixed:
- catching various previously uncatched errors

### Version 0.3

New:
- Command-line Interface

Fixed:
- lots of issues around opening / closing procedure

Known problems:
- Empty responses occasionally lead crashes

### Version 0.2

New:
- support for asynchronous acknowledgements
- promisification mostly done
- configuration inside `.env` file

Fixed:
- instabilities due to callbacks