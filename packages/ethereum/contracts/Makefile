# Use local foundry directory (for binaries)
FOUNDRY_DIR ?= ${CURDIR}/../../../.foundry

# add local Foundry install path (only once)
PATH := $(subst :${FOUNDRY_DIR}/bin,,$(PATH)):${FOUNDRY_DIR}/bin

SHELL := env $(shell which bash)

# utility wrapper for forge script to use same options on every invocation
forge := time forge
forge-bind = time forge bind
forge-script = env NETWORK=$(network) FOUNDRY_PROFILE=$(environment-type) ETHERSCAN_API_KEY=$(etherscan_api_key) time forge script

.PHONY: sc-test
sc-test:
	$(forge) test --gas-report

.PHONY: sc-coverage
sc-coverage:
	$(forge) coverage --report lcov && genhtml lcov.info -o report --branch-coverage && open report/index.html

.PHONY: sc-audit-coverage
sc-audit-coverage:
	$(forge) coverage --report lcov && genhtml lcov.info -o report --branch-coverage --legend --header-title "HOPR Smart Contracts" && open report/index.html

.PHONY: forked-local-anvil
forked-local-anvil:
	anvil -f 'https://provider-proxy.hoprnet.workers.dev/xdai_mainnet'

.PHONY: overwrite-sc-bindings
overwrite-sc-bindings: ## Only create types for Hopr* contracts
	$(forge-bind) --bindings-path ../crates/bindings --crate-name bindings \
		--overwrite --select '^Hopr.*?(Boost|[^t])$$'

.PHONY: check-sc-bindings
check-sc-bindings: ## Only create types for Hopr* contracts
	$(forge-bind) --bindings-path ./crates/bindings --root ./contracts \
		--crate-name bindings --select '^Hopr.*?(Boost|[^t])$$'

.PHONY: anvil-deploy-erc1820
anvil-deploy-erc1820: ensure-environment-and-network-are-set
anvil-deploy-erc1820: ## deploy ERC1820 contracts
	if [ -n "$(cast code 0x1820a4B7618BdE71Dce8cdc73aAB6C95905faD24)" ]; then \
	  echo "ERC1820 contract already deployed, skipping"; \
	else \
		cast send 0xa990077c3205cbDf861e17Fa532eeB069cE9fF96 --value 0.8ether --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 && \
		cast publish "0xf90a388085174876e800830c35008080b909e5608060405234801561001057600080fd5b506109c5806100206000396000f3fe608060405234801561001057600080fd5b50600436106100a5576000357c010000000000000000000000000000000000000000000000000000000090048063a41e7d5111610078578063a41e7d51146101d4578063aabbb8ca1461020a578063b705676514610236578063f712f3e814610280576100a5565b806329965a1d146100aa5780633d584063146100e25780635df8122f1461012457806365ba36c114610152575b600080fd5b6100e0600480360360608110156100c057600080fd5b50600160a060020a038135811691602081013591604090910135166102b6565b005b610108600480360360208110156100f857600080fd5b5035600160a060020a0316610570565b60408051600160a060020a039092168252519081900360200190f35b6100e06004803603604081101561013a57600080fd5b50600160a060020a03813581169160200135166105bc565b6101c26004803603602081101561016857600080fd5b81019060208101813564010000000081111561018357600080fd5b82018360208201111561019557600080fd5b803590602001918460018302840111640100000000831117156101b757600080fd5b5090925090506106b3565b60408051918252519081900360200190f35b6100e0600480360360408110156101ea57600080fd5b508035600160a060020a03169060200135600160e060020a0319166106ee565b6101086004803603604081101561022057600080fd5b50600160a060020a038135169060200135610778565b61026c6004803603604081101561024c57600080fd5b508035600160a060020a03169060200135600160e060020a0319166107ef565b604080519115158252519081900360200190f35b61026c6004803603604081101561029657600080fd5b508035600160a060020a03169060200135600160e060020a0319166108aa565b6000600160a060020a038416156102cd57836102cf565b335b9050336102db82610570565b600160a060020a031614610339576040805160e560020a62461bcd02815260206004820152600f60248201527f4e6f7420746865206d616e616765720000000000000000000000000000000000604482015290519081900360640190fd5b6103428361092a565b15610397576040805160e560020a62461bcd02815260206004820152601a60248201527f4d757374206e6f7420626520616e204552433136352068617368000000000000604482015290519081900360640190fd5b600160a060020a038216158015906103b85750600160a060020a0382163314155b156104ff5760405160200180807f455243313832305f4143434550545f4d4147494300000000000000000000000081525060140190506040516020818303038152906040528051906020012082600160a060020a031663249cb3fa85846040518363ffffffff167c01000000000000000000000000000000000000000000000000000000000281526004018083815260200182600160a060020a0316600160a060020a031681526020019250505060206040518083038186803b15801561047e57600080fd5b505afa158015610492573d6000803e3d6000fd5b505050506040513d60208110156104a857600080fd5b5051146104ff576040805160e560020a62461bcd02815260206004820181905260248201527f446f6573206e6f7420696d706c656d656e742074686520696e74657266616365604482015290519081900360640190fd5b600160a060020a03818116600081815260208181526040808320888452909152808220805473ffffffffffffffffffffffffffffffffffffffff19169487169485179055518692917f93baa6efbd2244243bfee6ce4cfdd1d04fc4c0e9a786abd3a41313bd352db15391a450505050565b600160a060020a03818116600090815260016020526040812054909116151561059a5750806105b7565b50600160a060020a03808216600090815260016020526040902054165b919050565b336105c683610570565b600160a060020a031614610624576040805160e560020a62461bcd02815260206004820152600f60248201527f4e6f7420746865206d616e616765720000000000000000000000000000000000604482015290519081900360640190fd5b81600160a060020a031681600160a060020a0316146106435780610646565b60005b600160a060020a03838116600081815260016020526040808220805473ffffffffffffffffffffffffffffffffffffffff19169585169590951790945592519184169290917f605c2dbf762e5f7d60a546d42e7205dcb1b011ebc62a61736a57c9089d3a43509190a35050565b600082826040516020018083838082843780830192505050925050506040516020818303038152906040528051906020012090505b92915050565b6106f882826107ef565b610703576000610705565b815b600160a060020a03928316600081815260208181526040808320600160e060020a031996909616808452958252808320805473ffffffffffffffffffffffffffffffffffffffff19169590971694909417909555908152600284528181209281529190925220805460ff19166001179055565b600080600160a060020a038416156107905783610792565b335b905061079d8361092a565b156107c357826107ad82826108aa565b6107b85760006107ba565b815b925050506106e8565b600160a060020a0390811660009081526020818152604080832086845290915290205416905092915050565b6000808061081d857f01ffc9a70000000000000000000000000000000000000000000000000000000061094c565b909250905081158061082d575080155b1561083d576000925050506106e8565b61084f85600160e060020a031961094c565b909250905081158061086057508015155b15610870576000925050506106e8565b61087a858561094c565b909250905060018214801561088f5750806001145b1561089f576001925050506106e8565b506000949350505050565b600160a060020a0382166000908152600260209081526040808320600160e060020a03198516845290915281205460ff1615156108f2576108eb83836107ef565b90506106e8565b50600160a060020a03808316600081815260208181526040808320600160e060020a0319871684529091529020549091161492915050565b7bffffffffffffffffffffffffffffffffffffffffffffffffffffffff161590565b6040517f01ffc9a7000000000000000000000000000000000000000000000000000000008082526004820183905260009182919060208160248189617530fa90519096909550935050505056fea165627a7a72305820377f4a2d4301ede9949f163f319021a6e9c687c292a5e2b2c4734c126b524e6c00291ba01820182018201820182018201820182018201820182018201820182018201820a01820182018201820182018201820182018201820182018201820182018201820"; \
	fi

# E.g. make anvil-deploy-safe-singleton network=anvil-localhost environment-type=local
.PHONY: anvil-deploy-safe-singleton
anvil-deploy-safe-singleton: ensure-environment-and-network-are-set
anvil-deploy-safe-singleton: ## deploy Safe Singleton contract according to https://github.com/safe-global/safe-singleton-factory/blob/main/artifacts/31337/deployment.json
	if [ -n "$(cast code 0x914d7Fec6aaC8cd542e72Bca78B30650d45643d7)" ]; then \
	  echo "Safe singleton contract already deployed, skipping"; \
	else \
	  	echo "Deploying Safe singleton"; \
		cast send 0xE1CB04A0fA36DdD16a06ea828007E35e1a3cBC37 --value 0.01ether --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 && \
		cast publish "0xf8a78085174876e800830186a08080b853604580600e600039806000f350fe7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe03601600081602082378035828234f58015156039578182fd5b8082525050506014600cf382f4f5a00dc4d1d21b308094a30f5f93da35e4d72e99115378f135f2295bea47301a3165a0636b822daad40aa8c52dd5132f378c0c0e6d83b4898228c7e21c84e631a0b891"; \
	fi

# E.g. make anvil-deploy-contracts network=anvil-localhost environment-type=local
.PHONY: anvil-deploy-contracts
anvil-deploy-contracts: ensure-environment-and-network-are-set
anvil-deploy-contracts: ## deploy HOPR contracts
	$(forge-script) --broadcast script/DeployAll.s.sol:DeployAllContractsScript

# E.g. make anvil-deploy-safe-suite environment-name=anvil-localhost environment-type=development
.PHONY: anvil-deploy-safe-suite
anvil-deploy-safe-suite: ensure-environment-and-network-are-set
anvil-deploy-safe-suite: ## deploy HOPR contracts
	$(forge-script) --broadcast script/SafeSuiteSetup.s.sol:SafeSuiteSetupScript

.PHONY: test-script
test-script: # development only.
	lsof -i :8545 -s TCP:LISTEN -t | xargs -I {} -n 1 kill {}
	anvil&
	$(MAKE) anvil-deploy-all network=anvil-localhost environment-type=local

.PHONY: anvil-deploy-all
anvil-deploy-all: ## deploy all contracts
	$(MAKE) anvil-deploy-erc1820 network=anvil-localhost environment-type=local
	$(MAKE) anvil-deploy-safe-singleton network=anvil-localhost environment-type=local
	$(MAKE) anvil-deploy-safe-suite network=anvil-localhost environment-type=local
	$(MAKE) anvil-deploy-contracts network=anvil-localhost environment-type=local

# E.g. make get-account-balances network=anvil-localhost environment-type=local account=0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266
.PHONY: get-account-balances
get-account-balances: ensure-environment-and-network-are-set
get-account-balances: ## Get native and token blances of an address
ifeq ($(account),)
	echo "parameter <account> missing" >&2 && exit 1
endif
	$(forge-script) script/GetAccountBalances.s.sol:GetAccountBalancesScript \
		--sig "run(address)" $(account)

# E.g. make request-funds network=anvil-localhost environment-type=local recipient=0x2402da10A6172ED018AEEa22CA60EDe1F766655C
.PHONY: request-funds
request-funds: ensure-environment-and-network-are-set ensure-privatekey-is-set
request-funds: ## Request 1000 xHOPR tokens for the recipient
ifeq ($(recipient),)
	echo "parameter <recipient> missing" >&2 && exit 1
endif
	$(forge-script) script/SingleAction.s.sol:SingleActionFromPrivateKeyScript \
		--broadcast --sig "mintXHopr(address,uint256)" $(recipient) 1000

# E.g. make faucet network=anvil-localhost environment-type=local recipient=0x2402da10A6172ED018AEEa22CA60EDe1F766655C hopramount=100000000000000000000 nativeamount=10000000000000000000
# E.g. make faucet network=anvil-localhost environment-type=local recipient=0x2402da10A6172ED018AEEa22CA60EDe1F766655C nativeamount=10000000000000000000
# E.g. make faucet network=anvil-localhost environment-type=local recipient=0x2402da10A6172ED018AEEa22CA60EDe1F766655C hopramount=100000000000000000000
.PHONY: faucet
faucet: ensure-environment-and-network-are-set ensure-privatekey-is-set
faucet: ## Mint some HOPR tokens to the desired amount (default: 20000) and send some native tokens to the recipient until it reaches certain hopramount (default value: 10). Token value should be in 18 decimals
ifeq ($(recipient),)
	echo "parameter <recipient> missing" >&2 && exit 1
endif
	$(forge-script) script/SingleAction.s.sol:SingleActionFromPrivateKeyScript \
        --broadcast --sig "transferOrMintHoprAndSendNativeToAmount(address,uint256,uint256)" \
        $(recipient) $(or $(hopramount),20000000000000000000000) $(or $(nativeamount),10000000000000000000)

# E.g. make request-nrnft network=anvil-localhost environment-type=local recipient=0x2402da10A6172ED018AEEa22CA60EDe1F766655C nftrank=developer
.PHONY: request-nrnft
request-nrnft: ensure-environment-and-network-are-set ensure-privatekey-is-set
request-nrnft: ## Request one HoprBoost Network_registry NFT for the recipient given it has none and hasn't staked Network_registry NFT
ifeq ($(recipient),)
	echo "parameter <recipient> missing" >&2 && exit 1
endif
ifeq ($(nftrank),)
	echo "parameter <nftrank> missing, it can be either 'developer' or 'community'" >&2 && exit 1
endif
	$(forge-script) script/SingleAction.s.sol:SingleActionFromPrivateKeyScript \
		--broadcast --sig "transferNetworkRegistryNft(address,string)" \
		$(recipient) $(nftrank)

# E.g. make express-setup-safe-module network=anvil-localhost environment-type=local node_address="0xcb192a4ba9abea8579c59231b8a57e3f0591e02b" hopramount=1000000000000000000 nativeamount=1000000000000000000
.PHONY: express-setup-safe-module
express-setup-safe-module: ensure-environment-and-network-are-set ensure-privatekey-is-set
express-setup-safe-module: ## create a new safe and a new node management module, include the provided node address
ifeq ($(node_address),)
	echo "parameter <node_address> missing" >&2 && exit 1
endif
	$(forge-script) script/SingleAction.s.sol:SingleActionFromPrivateKeyScript \
		--broadcast --sig "expressSetupSafeModule(address[],uint256,uint256)" [$(node_address)]  $(or $(hopramount),20000000000000000000000) $(or $(nativeamount),10000000000000000000)

# E.g. make migrate-safe-module network=anvil-localhost environment-type=local node_address="0xcb192a4ba9abea8579c59231b8a57e3f0591e02b" safe_address=0x3A0A80BD107A4E029154286293296e62d235a4ce module_address=0xcE594ED193F314d66bC3Af9b53930CD9Ea87cc6E
.PHONY: migrate-safe-module
migrate-safe-module: ensure-environment-and-network-are-set ensure-privatekey-is-set
migrate-safe-module: ## Migate existing node, safe and module to a new network
ifeq ($(node_address),)
	echo "parameter <node_address> missing" >&2 && exit 1
endif
ifeq ($(safe_address),)
	echo "parameter <safe_address> missing" >&2 && exit 1
endif
ifeq ($(module_address),)
	echo "parameter <module_address> missing" >&2 && exit 1
endif
	$(forge-script) script/SingleAction.s.sol:SingleActionFromPrivateKeyScript \
		--broadcast --sig "migrateSafeModule(address[],address,address)" [$(node_address)] $(safe_address) $(module_address)

# E.g. make add-all-allowed-target network=anvil-localhost environment-type=local target_address=0xcb192a4ba9abea8579c59231b8a57e3f0591e02b safe_address=0x3A0A80BD107A4E029154286293296e62d235a4ce module_address=0xcE594ED193F314d66bC3Af9b53930CD9Ea87cc6E
.PHONY: add-all-allowed-target
add-all-allowed-target: ensure-environment-and-network-are-set ensure-privatekey-is-set
add-all-allowed-target: ## create a new safe and a new node management module, include the provided node address
ifeq ($(target_address),)
	echo "parameter <target_address> missing" >&2 && exit 1
endif
ifeq ($(safe_address),)
	echo "parameter <safe_address> missing" >&2 && exit 1
endif
ifeq ($(module_address),)
	echo "parameter <module_address> missing" >&2 && exit 1
endif
	$(forge-script) script/SingleAction.s.sol:SingleActionFromPrivateKeyScript \
		--broadcast --sig "addAllAllowedTargetToModuleBySafe(address,address,address)" $(target_address) $(safe_address) $(module_address)

# E.g. make stake-funds network=anvil-localhost environment-type=local
.PHONY: stake-funds
stake-funds: ensure-environment-and-network-are-set ensure-privatekey-is-set
stake-funds: ## stake funds (idempotent operation)
	$(forge-script) script/SingleAction.s.sol:SingleActionFromPrivateKeyScript \
		--broadcast --sig "stakeXHopr(uint256)" 1000000000000000000000

# E.g. make stake-nrnft network=anvil-localhost environment-type=local nftrank=developer
.PHONY: stake-nrnft
stake-nrnft: ensure-environment-and-network-are-set ensure-privatekey-is-set
stake-nrnft: ## stake Network_registry NFTs (idempotent operation)
ifeq ($(nftrank),)
	echo "parameter <nftrank> missing, it can be either 'developer' or 'community'" >&2 && exit 1
endif
	$(forge-script) script/SingleAction.s.sol:SingleActionFromPrivateKeyScript \
		--broadcast --sig "stakeNetworkRegistryNft(string)" $(nftrank)

# E.g. make register-nodes network=anvil-localhost environment-type=local staking_addresses="[0x3A0A80BD107A4E029154286293296e62d235a4ce,0x4AAf51e0b43d8459AF85E33eEf3Ffb7EACb5532C]" node_addresses="[0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266,0x70997970c51812dc3a010c7d01b50e0d17dc79c8]"
.PHONY: register-nodes
register-nodes: ensure-environment-and-network-are-set ensure-privatekey-is-set
register-nodes: ## manager registers nodes and safes in network registry contract
ifeq ($(staking_addresses),)
	echo "parameter <staking_addresses> missing" >&2 && exit 1
endif
ifeq ($(node_addresses),)
	echo "parameter <node_addresses> missing" >&2 && exit 1
endif
	$(forge-script) script/SingleAction.s.sol:SingleActionFromPrivateKeyScript \
		--broadcast --sig "registerNodes(address[],address[])" \
		$(staking_addresses) $(node_addresses)

# E.g. make deregister-nodes network=anvil-localhost environment-type=local node_addresses="[0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266,0x70997970c51812dc3a010c7d01b50e0d17dc79c8]"
.PHONY: deregister-nodes
deregister-nodes: ensure-environment-and-network-are-set ensure-privatekey-is-set
deregister-nodes: ## owner deregister given nodes from network registry contract
ifeq ($(node_addresses),)
	echo "parameter <node_addresses> missing" >&2 && exit 1
endif
	$(forge-script) script/SingleAction.s.sol:SingleActionFromPrivateKeyScript \
		--broadcast --sig "deregisterNodes(address[])" \
		$(node_addresses)

# E.g. make enable-network-registry network=anvil-localhost environment-type=local
.PHONY: enable-network-registry
enable-network-registry: ensure-environment-and-network-are-set ensure-privatekey-is-set
enable-network-registry: ## owner enables network registry (smart contract) globally
	$(forge-script) script/SingleAction.s.sol:SingleActionFromPrivateKeyScript \
		--broadcast --sig "enableNetworkRegistry()"

# E.g. make disable-network-registry network=anvil-localhost environment-type=local
.PHONY: disable-network-registry
disable-network-registry: ensure-environment-and-network-are-set ensure-privatekey-is-set
disable-network-registry: ## owner disables network registry (smart contract) globally
	$(forge-script) script/SingleAction.s.sol:SingleActionFromPrivateKeyScript \
		--broadcast --sig "disableNetworkRegistry()"

# E.g. make sync-eligibility network=anvil-localhost environment-type=local staking_addresses="[0x3A0A80BD107A4E029154286293296e62d235a4ce,0x4AAf51e0b43d8459AF85E33eEf3Ffb7EACb5532C]"
.PHONY: sync-eligibility
sync-eligibility: ensure-environment-and-network-are-set ensure-privatekey-is-set
sync-eligibility: ## owner sync eligibility of peers
ifeq ($(staking_addresses),)
	echo "parameter <staking_addresses> missing" >&2 && exit 1
endif
	$(forge-script) script/SingleAction.s.sol:SingleActionFromPrivateKeyScript \
		--broadcast --sig "syncEligibility(address[])" $(staking_addresses)

# These targets needs to be splitted in macOs systems
ensure-environment-and-network-are-set: ensure-network-is-set ensure-environment-is-set ensure-etherscan-api-key-is-set

ensure-network-is-set:
ifeq ($(network),)
	echo "parameter <network> missing" >&2 && exit 1
else
environment-type != jq '.networks."$(network)".environment_type // empty' contracts-addresses.json
endif

.PHONY: ensure-etherscan-api-key-is-set
ensure-etherscan-api-key-is-set:
ifeq ($(origin ETHERSCAN_API_KEY),undefined)
# On local testing a dummy key is needed
etherscan_api_key=dummyKey
else
etherscan_api_key=${ETHERSCAN_API_KEY}
endif

ensure-environment-is-set:
ifeq ($(environment-type),)
	echo "could not read environment info from contracts-addresses.json" >&2 && exit 1
endif

ensure-privatekey-is-set:
ifeq ($(origin PRIVATE_KEY),undefined)
	echo "<PRIVATE_KEY> environment variable missing" >&2 && exit 1
endif
