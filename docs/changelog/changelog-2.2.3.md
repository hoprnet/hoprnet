Below there is a list with the contents of this release

### 🐛 Bugfix

- #6938 - db: Fix ticket aggregation balance threshold
- #6945 - p2p: Fall back to TCP on dappnode
- #6942 - Improve eligibility check by removing type conversions
- #6930 - db: Fix channel/ticket state mismatch on startup
- #6895 - Fix incorrectly filtered out offline peers and other tweaks in Promiscuous strategy defaults
- #6873 - Fix the cache cleaning of multiaddresses on peer dial
- #6872 - Set the aggregation ticket threshold to the middle of configuration range

### ⚡ Other

- #6934 - ci: Add harden-runner to all workflows
- #6925 - tests: Fix per-test teardown
- #6898 - Add initial delay and improve peer pings handling in strategy
- #6892 - Update defaults of Promiscuous strategy
- #6891 - Remove multiaddress drop on request response
- #6889 - Improve channel graph node scoring
- #6887 - Delete closed foreign channels from the database
- #6885 - Updates to Promiscuous strategy
- #6884 - docker: Fix entrypoint script
- #6882 - db: Add more indices
- #6875 - swarm: Add network info logs
- #6858 - Path selector refactoring
- #6809 - Update libp2p to 0.55
