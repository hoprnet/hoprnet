Below there is a list with the contents of this release

### 🚀 New Features

- #6059 - Decrease the scope of some RwLocks
- #6058 - Soft-restart indexer loop on error, expose `max_block_range` on the CLI
- #6052 - Change batch size default and small config refactoring
- #6047 - Documentation updates and minor code cleanup
- #6042 - docker: Add bash to docker images
- #6033 - Use smart default wherever reasonable

### 🐛 Bug

- #6057 - Terminate Indexer processing loop on hard failure
- #6037 - Fixes for issues found in the hopr-sdk testing

### ⚡ Other

- #6056 - Enforce strict configuration file parsing to deny unknown keys
- #6055 - Relax the limitation to strictly check existence of the hoprd identity from the configuration
- #6048 - Add documentation in hopli
- #6046 - Improve documentation across different crates
- #6029 - Update the nix environment and the package lock
- #6027 - Allow websockets to parse the auth from a websocket protocol
- #5902 - Rust docs improvements
