Below there is a list with the contents of this release

### 🚀 New Features

- #6166 - Add log information when node gets registered
- #6129 - Improve chain event processing from logs
- #6113 - Fix return code for set alias API call
- #6096 - Add DB API traits and implementations
- #6092 - Introduce ORM and database migrations
- #6090 - Logging and db scoping fixes
- #6084 - Auto-redeem single tickets of value on closure initiation
- #6079 - Use exactly the amount of hops specified by the caller
- #6074 - Add more logs to debug double transaction transmission in mempool
- #6071 - Use Sqlite backend for Network peers storage to prevent high-level locking
- #6067 - Separate `get_peer_with_quality` expression
- #6066 - Improve visbility into code hot paths using metrics and logs
- #6065 - Better handling of missing block errors

### 🐛 Bug

- #6174 - Workaround version matching issue in Promiscuous strategy
- #6171 - Fix incorrect VRF signer in Ticket Aggregation
- #6162 - Deprecate CLI configuration of strategies
- #6150 - Reorder multistrategy to first start to happen only once the node is fully running
- #6114 - fix: missing serialization info in `ClosureFinalizerStrategyConfig`
- #6100 - Fix commit hash labeling docker images
- #6089 - Fix multistrategy not defaulting values for hoprd config properly
- #6088 - Collection of fixes to scoping and locks for the DB

### ⚡ Other

- #6167 - hopli: Configure contracts root in docker image by default
- #6165 - Fix error codes on certain API endpoints
- #6151 - Reduce hopli default provider tx poll interval from 7s to 10 ms
- #6145 - Fix minor logging issues to make the info output more readable
- #6135 - Improvements to ticket metrics and deprecation of ticket displaying API endpoints
- #6131 - Add node configuration endpoint
- #6124 - Saint Louis fixes from practical testing session
- #6119 - Add support for PR labeling
- #6107 - `chain-actions` and `core-strategy` migration to the new DB
- #6069 - Remove `Address::random()` function from production code
- #6063 - Fix the hoprd-api-schema generation by directly executing the binary
- #6062 - Replace the log infrastructure with tracing
