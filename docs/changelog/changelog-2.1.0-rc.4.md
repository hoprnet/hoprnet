Below there is a list with the contents of this release

### 🚀 New Features

- #6226 - Do not use ethers-rs pagination of `eth_getLogs`
- #6220 - Increase size of the DB connection pool
- #6203 - Squash additional migrations to prior ones
- #6198 - Add metrics for logs processed by the Indexer
- #6197 - Add log transaction checksum into Indexer
- #6181 - Add PeerID <-> Public key converter to hopli

### 🐛 Bug

- #6212 - Node cannot not start after synced due to a failure getting connection pool
- #6237 - The `/node/peers` API endpoint sometimes returned empty multiaddreses as `announced`
- #6235 - Path finding should only consider nodes that proven to be reliable
- #6230 - Fail hard on failure to load the block number from RPC provider
- #6227 - chain: do not fetch logs if no topics are set
- #6218 - Fix Indexer checksum computation
- #6215 - Make set_alias e2e test stable again
- #6211 - Fix off-by-1 in the Indexer
- #6206 - Do not allow duplicate peer ID aliasing
- #6202 - Fix comparison of invalid balance types
- #6188 - Fix updating node info table

### ⚡ Other

- #6244 - Update metrics
- #6242 - Add more connection pool parameters
- #6238 - Increase the connection pool size of Peers DB
- #6234 - Update checksum printout
- #6233 - Fix subtraction operations on Durations to be saturating
- #6223 - Log RPC request/response JSON
- #6213 - Improves the aliases e2e test
- #6210 - Improve descriptions in the 2.1 API docs
- #6201 - Add commit hash to log output in hoprd
- #6194 - Fix hoprd persistence directory paths
- #6193 - Allow RPC provider to feed past blocks
- #6192 - Add support for yamux as default mux to libp2p swarm
- #6190 - Fix issues with alias urldecoding
- #6189 - Fix the display of sync metric
