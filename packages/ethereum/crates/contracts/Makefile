sc-test:
	forge test --gas-report

sc-coverage:
	forge coverage --report lcov && genhtml lcov.info -o report --branch-coverage && open report/index.html

forked-local-anvil:
	anvil -f 'https://provider-proxy.hoprnet.workers.dev/xdai_mainnet'

overwrite-sc-bindings:
	forge bind --bindings-path ../bindings --crate-name bindings --overwrite