# Use local foundry directory (for binaries)
FOUNDRY_DIR ?= ${CURDIR}/../../../.foundry

# add local Foundry install path (only once)
PATH := $(subst :${FOUNDRY_DIR}/bin,,$(PATH)):${FOUNDRY_DIR}/bin

# use custom PATH in all shell processes
SHELL := env PATH=$(PATH) $(shell which bash)

# utility wrapper for forge script to use same options on every invocation
forge := time forge
forge-bind = time forge bind
forge-script = env ENVIRONMENT_NAME=$(environment-name) FOUNDRY_PROFILE=$(environment-type) time forge script

.PHONY: sc-test
sc-test:
	$(forge) test --gas-report

.PHONY: sc-coverage
sc-coverage:
	$(forge) coverage --report lcov && genhtml lcov.info -o report --branch-coverage && open report/index.html

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
anvil-deploy-erc1820: ensure-environment-is-set
anvil-deploy-erc1820: ## deploy ERC1820 contracts
	if [ -n "$(cast code 0x1820a4B7618BdE71Dce8cdc73aAB6C95905faD24)" ]; then \
	  echo "ERC1820 contract already deployed, skipping"; \
	else \
		cast send 0xa990077c3205cbDf861e17Fa532eeB069cE9fF96 --value 0.8ether --from 0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266 && \
		cast publish "0xf90a388085174876e800830c35008080b909e5608060405234801561001057600080fd5b506109c5806100206000396000f3fe608060405234801561001057600080fd5b50600436106100a5576000357c010000000000000000000000000000000000000000000000000000000090048063a41e7d5111610078578063a41e7d51146101d4578063aabbb8ca1461020a578063b705676514610236578063f712f3e814610280576100a5565b806329965a1d146100aa5780633d584063146100e25780635df8122f1461012457806365ba36c114610152575b600080fd5b6100e0600480360360608110156100c057600080fd5b50600160a060020a038135811691602081013591604090910135166102b6565b005b610108600480360360208110156100f857600080fd5b5035600160a060020a0316610570565b60408051600160a060020a039092168252519081900360200190f35b6100e06004803603604081101561013a57600080fd5b50600160a060020a03813581169160200135166105bc565b6101c26004803603602081101561016857600080fd5b81019060208101813564010000000081111561018357600080fd5b82018360208201111561019557600080fd5b803590602001918460018302840111640100000000831117156101b757600080fd5b5090925090506106b3565b60408051918252519081900360200190f35b6100e0600480360360408110156101ea57600080fd5b508035600160a060020a03169060200135600160e060020a0319166106ee565b6101086004803603604081101561022057600080fd5b50600160a060020a038135169060200135610778565b61026c6004803603604081101561024c57600080fd5b508035600160a060020a03169060200135600160e060020a0319166107ef565b604080519115158252519081900360200190f35b61026c6004803603604081101561029657600080fd5b508035600160a060020a03169060200135600160e060020a0319166108aa565b6000600160a060020a038416156102cd57836102cf565b335b9050336102db82610570565b600160a060020a031614610339576040805160e560020a62461bcd02815260206004820152600f60248201527f4e6f7420746865206d616e616765720000000000000000000000000000000000604482015290519081900360640190fd5b6103428361092a565b15610397576040805160e560020a62461bcd02815260206004820152601a60248201527f4d757374206e6f7420626520616e204552433136352068617368000000000000604482015290519081900360640190fd5b600160a060020a038216158015906103b85750600160a060020a0382163314155b156104ff5760405160200180807f455243313832305f4143434550545f4d4147494300000000000000000000000081525060140190506040516020818303038152906040528051906020012082600160a060020a031663249cb3fa85846040518363ffffffff167c01000000000000000000000000000000000000000000000000000000000281526004018083815260200182600160a060020a0316600160a060020a031681526020019250505060206040518083038186803b15801561047e57600080fd5b505afa158015610492573d6000803e3d6000fd5b505050506040513d60208110156104a857600080fd5b5051146104ff576040805160e560020a62461bcd02815260206004820181905260248201527f446f6573206e6f7420696d706c656d656e742074686520696e74657266616365604482015290519081900360640190fd5b600160a060020a03818116600081815260208181526040808320888452909152808220805473ffffffffffffffffffffffffffffffffffffffff19169487169485179055518692917f93baa6efbd2244243bfee6ce4cfdd1d04fc4c0e9a786abd3a41313bd352db15391a450505050565b600160a060020a03818116600090815260016020526040812054909116151561059a5750806105b7565b50600160a060020a03808216600090815260016020526040902054165b919050565b336105c683610570565b600160a060020a031614610624576040805160e560020a62461bcd02815260206004820152600f60248201527f4e6f7420746865206d616e616765720000000000000000000000000000000000604482015290519081900360640190fd5b81600160a060020a031681600160a060020a0316146106435780610646565b60005b600160a060020a03838116600081815260016020526040808220805473ffffffffffffffffffffffffffffffffffffffff19169585169590951790945592519184169290917f605c2dbf762e5f7d60a546d42e7205dcb1b011ebc62a61736a57c9089d3a43509190a35050565b600082826040516020018083838082843780830192505050925050506040516020818303038152906040528051906020012090505b92915050565b6106f882826107ef565b610703576000610705565b815b600160a060020a03928316600081815260208181526040808320600160e060020a031996909616808452958252808320805473ffffffffffffffffffffffffffffffffffffffff19169590971694909417909555908152600284528181209281529190925220805460ff19166001179055565b600080600160a060020a038416156107905783610792565b335b905061079d8361092a565b156107c357826107ad82826108aa565b6107b85760006107ba565b815b925050506106e8565b600160a060020a0390811660009081526020818152604080832086845290915290205416905092915050565b6000808061081d857f01ffc9a70000000000000000000000000000000000000000000000000000000061094c565b909250905081158061082d575080155b1561083d576000925050506106e8565b61084f85600160e060020a031961094c565b909250905081158061086057508015155b15610870576000925050506106e8565b61087a858561094c565b909250905060018214801561088f5750806001145b1561089f576001925050506106e8565b506000949350505050565b600160a060020a0382166000908152600260209081526040808320600160e060020a03198516845290915281205460ff1615156108f2576108eb83836107ef565b90506106e8565b50600160a060020a03808316600081815260208181526040808320600160e060020a0319871684529091529020549091161492915050565b7bffffffffffffffffffffffffffffffffffffffffffffffffffffffff161590565b6040517f01ffc9a7000000000000000000000000000000000000000000000000000000008082526004820183905260009182919060208160248189617530fa90519096909550935050505056fea165627a7a72305820377f4a2d4301ede9949f163f319021a6e9c687c292a5e2b2c4734c126b524e6c00291ba01820182018201820182018201820182018201820182018201820182018201820a01820182018201820182018201820182018201820182018201820182018201820"; \
	fi

# E.g. make anvil-deploy-contracts environment-name=anvil-localhost environment-type=development
.PHONY: anvil-deploy-contracts
anvil-deploy-contracts: ensure-environment-is-set
anvil-deploy-contracts: ## deploy HOPR contracts
	$(forge-script) --broadcast script/DeployAll.s.sol:DeployAllContractsScript

.PHONY: anvil-deploy-all
anvil-deploy-all: ## deploy all contracts
	$(MAKE) anvil-deploy-erc1820 environment-name=anvil-localhost environment-type=development
	$(MAKE) anvil-deploy-contracts environment-name=anvil-localhost environment-type=development

# E.g. make get-account-balances environment-name=anvil-localhost environment-type=development account=0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266
.PHONY: get-account-balances
get-account-balances: ensure-environment-is-set
get-account-balances: ## Get native and token blances of an address
ifeq ($(account),)
	echo "parameter <account> missing" >&2 && exit 1
endif
	$(forge-script) script/GetAccountBalances.s.sol:GetAccountBalancesScript \
		--sig "run(address)" $(account)

# E.g. make request-funds environment-name=anvil-localhost environment-type=development recipient=0x2402da10A6172ED018AEEa22CA60EDe1F766655C
.PHONY: request-funds
request-funds: ensure-environment-is-set ensure-privatekey-is-set
request-funds: ## Request 1000 xHOPR tokens for the recipient
ifeq ($(recipient),)
	echo "parameter <recipient> missing" >&2 && exit 1
endif
	$(forge-script) script/SingleAction.s.sol:SingleActionFromPrivateKeyScript \
		--broadcast --sig "mintXHopr(address,uint256)" $(recipient) 1000

# E.g. make faucet environment-name=anvil-localhost environment-type=development recipient=0x2402da10A6172ED018AEEa22CA60EDe1F766655C hopramount=100000000000000000000 nativeamount=10000000000000000000
# E.g. make faucet environment-name=anvil-localhost environment-type=development recipient=0x2402da10A6172ED018AEEa22CA60EDe1F766655C nativeamount=10000000000000000000
# E.g. make faucet environment-name=anvil-localhost environment-type=development recipient=0x2402da10A6172ED018AEEa22CA60EDe1F766655C hopramount=100000000000000000000
.PHONY: faucet
faucet: ensure-environment-is-set ensure-privatekey-is-set
faucet: ## Mint some (default: 20000) HOPR tokens and send some (default value: 10) native tokens to the recipient hopramount. Token value should be in 18 decimals
ifeq ($(recipient),)
	echo "parameter <recipient> missing" >&2 && exit 1
endif
	$(forge-script) script/SingleAction.s.sol:SingleActionFromPrivateKeyScript \
        --broadcast --sig "transferOrMintHoprAndSendNative(address,uint256,uint256)" \
        $(recipient) $(or $(hopramount),20000000000000000000000) $(or $(nativeamount),10000000000000000000)

# E.g. make request-nrnft environment-name=anvil-localhost environment-type=development recipient=0x2402da10A6172ED018AEEa22CA60EDe1F766655C nftrank=developer
.PHONY: request-nrnft
request-nrnft: ensure-environment-is-set ensure-privatekey-is-set
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

# E.g. make stake-funds environment-name=anvil-localhost environment-type=development
.PHONY: stake-funds
stake-funds: ensure-environment-is-set ensure-privatekey-is-set
stake-funds: ## stake funds (idempotent operation)
	$(forge-script) script/SingleAction.s.sol:SingleActionFromPrivateKeyScript \
		--broadcast --sig "stakeXHopr(uint256)" 1000000000000000000000

# E.g. make stake-nrnft environment-name=anvil-localhost environment-type=development nftrank=developer
.PHONY: stake-nrnft
stake-nrnft: ensure-environment-is-set ensure-privatekey-is-set
stake-nrnft: ## stake Network_registry NFTs (idempotent operation)
ifeq ($(nftrank),)
	echo "parameter <nftrank> missing, it can be either 'developer' or 'community'" >&2 && exit 1
endif
	$(forge-script) script/SingleAction.s.sol:SingleActionFromPrivateKeyScript \
		--broadcast --sig "stakeNetworkRegistryNft(string)" $(nftrank)

# E.g. make self-register-node environment-name=anvil-localhost environment-type=development peer_ids="[16Uiu2HAmAmXnq8FtFsBAnSyfujaXbU3B5wdVRPYGfN8S99E5azdH,16Uiu2HAky1FmbdyrY4EC2ZmKVH5t1XTqZqkutnqNf5BpSjHuD9nF]"
.PHONY: self-register-node
self-register-node: ensure-environment-is-set ensure-privatekey-is-set
self-register-node: ## staker register a node in network registry contract
ifeq ($(peer_ids),)
	echo "parameter <peer_ids> missing" >&2 && exit 1
endif
	$(forge-script) script/SingleAction.s.sol:SingleActionFromPrivateKeyScript \
		--broadcast --sig "selfRegisterNodes(string[])" [$(peer_ids)] 


# E.g. make self-deregister-node environment-name=anvil-localhost environment-type=development peer_ids="[16Uiu2HAmAmXnq8FtFsBAnSyfujaXbU3B5wdVRPYGfN8S99E5azdH,16Uiu2HAky1FmbdyrY4EC2ZmKVH5t1XTqZqkutnqNf5BpSjHuD9nF]"
.PHONY: self-deregister-node
self-deregister-node: ensure-environment-is-set ensure-privatekey-is-set
self-deregister-node: ## staker deregister a node in network registry contract
ifeq ($(peer_ids),)
	echo "parameter <peer_ids> missing" >&2 && exit 1
endif
	$(forge-script) script/SingleAction.s.sol:SingleActionFromPrivateKeyScript \
		--broadcast --sig "selfDeregisterNodes(string[])" $(peer_ids)

# E.g. make register-nodes environment-name=anvil-localhost environment-type=development staking_addresses="[0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266,0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266]" peer_ids="[16Uiu2HAmAmXnq8FtFsBAnSyfujaXbU3B5wdVRPYGfN8S99E5azdH,16Uiu2HAky1FmbdyrY4EC2ZmKVH5t1XTqZqkutnqNf5BpSjHuD9nF]"
.PHONY: register-nodes
register-nodes: ensure-environment-is-set ensure-privatekey-is-set
register-nodes: ## owner register given nodes in network registry contract
ifeq ($(peer_ids),)
	echo "parameter <peer_ids> missing" >&2 && exit 1
endif
ifeq ($(staking_addresses),)
	echo "parameter <staking_addresses> missing" >&2 && exit 1
endif
	$(forge-script) script/SingleAction.s.sol:SingleActionFromPrivateKeyScript \
		--broadcast --sig "registerNodes(address[],string[])" \
		$(staking_addresses) $(peer_ids)

# E.g. make deregister-nodes environment-name=anvil-localhost environment-type=development staking_addresses="[0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266]" peer_ids="[16Uiu2HAmAmXnq8FtFsBAnSyfujaXbU3B5wdVRPYGfN8S99E5azdH,16Uiu2HAky1FmbdyrY4EC2ZmKVH5t1XTqZqkutnqNf5BpSjHuD9nF]"
.PHONY: deregister-nodes
deregister-nodes: ensure-environment-is-set ensure-privatekey-is-set
deregister-nodes: ## owner deregister given nodes from network registry contract
ifeq ($(peer_ids),)
	echo "parameter <peer_ids> missing" >&2 && exit 1
endif
ifeq ($(staking_addresses),)
	echo "parameter <staking_addresses> missing. Please provide an empty array "[]" if no value of staking address should be provided." >&2 && exit 1
endif
	$(forge-script) script/SingleAction.s.sol:SingleActionFromPrivateKeyScript \
		--broadcast --sig "deregisterNodes(address[],string[])" \
		$(staking_addresses) $(peer_ids)

# E.g. make enable-network-registry environment-name=anvil-localhost environment-type=development
.PHONY: enable-network-registry
enable-network-registry: ensure-environment-is-set ensure-privatekey-is-set
enable-network-registry: ## owner enables network registry (smart contract) globally
	$(forge-script) script/SingleAction.s.sol:SingleActionFromPrivateKeyScript \
		--broadcast --sig "enableNetworkRegistry()"

# E.g. make disable-network-registry environment-name=anvil-localhost environment-type=development
.PHONY: disable-network-registry
disable-network-registry: ensure-environment-is-set ensure-privatekey-is-set
disable-network-registry: ## owner disables network registry (smart contract) globally
	$(forge-script) script/SingleAction.s.sol:SingleActionFromPrivateKeyScript \
		--broadcast --sig "disableNetworkRegistry()"

# E.g. make force-eligibility-update environment-name=anvil-localhost environment-type=development staking_addresses="[0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266,0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266]" eligibility="[true,false]"
.PHONY: force-eligibility-update
force-eligibility-update: ensure-environment-is-set ensure-privatekey-is-set
force-eligibility-update: ## owner forces eligibility update
ifeq ($(staking_addresses),)
	echo "parameter <staking_addresses> missing" >&2 && exit 1
endif
ifeq ($(eligibility),)
	echo "parameter <eligibility> missing" >&2 && exit 1
endif
	$(forge-script) script/SingleAction.s.sol:SingleActionFromPrivateKeyScript \
		--broadcast --sig "forceEligibilityUpdate(address[],bool[])" \
		$(staking_addresses) $(eligibility)

# E.g. make sync-eligibility environment-name=anvil-localhost environment-type=development peer_ids="[16Uiu2HAmAmXnq8FtFsBAnSyfujaXbU3B5wdVRPYGfN8S99E5azdH,16Uiu2HAky1FmbdyrY4EC2ZmKVH5t1XTqZqkutnqNf5BpSjHuD9nF]"
.PHONY: sync-eligibility
sync-eligibility: ensure-environment-is-set ensure-privatekey-is-set
sync-eligibility: ## owner sync eligibility of peers
ifeq ($(peer_ids),)
	echo "parameter <peer_ids> missing" >&2 && exit 1
endif
	$(forge-script) script/SingleAction.s.sol:SingleActionFromPrivateKeyScript \
		--broadcast --sig "syncEligibility(string[])" $(peer_ids)

ensure-environment-is-set:
ifeq ($(environment-name),)
	echo "parameter <environment-name> missing" >&2 && exit 1
else
environment-type != jq '.environments."$(environment-name)".environment_type // empty' contracts-addresses.json
ifeq ($(environment-type),)
	echo "could not read environment info from contracts-addresses.json" >&2 && exit 1
endif
endif

ensure-privatekey-is-set:
ifeq ($(origin PRIVATE_KEY),undefined)
	echo "<PRIVATE_KEY> environment variable missing" >&2 && exit 1
endif
