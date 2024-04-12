SHELL := env $(shell which bash)

# utility wrapper for forge script to use same options on every invocation
forge := time forge
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
	anvil -f 'https://gnosis-provider.rpch.tech/'

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

# E.g. make anvil-deploy-multicall3 environment-name=anvil-localhost environment-type=development
.PHONY: anvil-deploy-multicall3
anvil-deploy-multicall3: ensure-environment-and-network-are-set
anvil-deploy-multicall3: ## deploy HOPR contracts
	if [ -n "$(cast code 0xcA11bde05977b3631167028862bE2a173976CA11)" ]; then \
	  echo "Multicall3 contract already deployed, skipping"; \
	else \
    echo "Deploying Safe singleton"; \
		cast send 0x05f32b3cc3888453ff71b01135b34ff8e41263f2 --value 0.1ether --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 && \
		cast publish "0xf90f538085174876e800830f42408080b90f00608060405234801561001057600080fd5b50610ee0806100206000396000f3fe6080604052600436106100f35760003560e01c80634d2301cc1161008a578063a8b0574e11610059578063a8b0574e1461025a578063bce38bd714610275578063c3077fa914610288578063ee82ac5e1461029b57600080fd5b80634d2301cc146101ec57806372425d9d1461022157806382ad56cb1461023457806386d516e81461024757600080fd5b80633408e470116100c65780633408e47014610191578063399542e9146101a45780633e64a696146101c657806342cbb15c146101d957600080fd5b80630f28c97d146100f8578063174dea711461011a578063252dba421461013a57806327e86d6e1461015b575b600080fd5b34801561010457600080fd5b50425b6040519081526020015b60405180910390f35b61012d610128366004610a85565b6102ba565b6040516101119190610bbe565b61014d610148366004610a85565b6104ef565b604051610111929190610bd8565b34801561016757600080fd5b50437fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0140610107565b34801561019d57600080fd5b5046610107565b6101b76101b2366004610c60565b610690565b60405161011193929190610cba565b3480156101d257600080fd5b5048610107565b3480156101e557600080fd5b5043610107565b3480156101f857600080fd5b50610107610207366004610ce2565b73ffffffffffffffffffffffffffffffffffffffff163190565b34801561022d57600080fd5b5044610107565b61012d610242366004610a85565b6106ab565b34801561025357600080fd5b5045610107565b34801561026657600080fd5b50604051418152602001610111565b61012d610283366004610c60565b61085a565b6101b7610296366004610a85565b610a1a565b3480156102a757600080fd5b506101076102b6366004610d18565b4090565b60606000828067ffffffffffffffff8111156102d8576102d8610d31565b60405190808252806020026020018201604052801561031e57816020015b6040805180820190915260008152606060208201528152602001906001900390816102f65790505b5092503660005b8281101561047757600085828151811061034157610341610d60565b6020026020010151905087878381811061035d5761035d610d60565b905060200281019061036f9190610d8f565b6040810135958601959093506103886020850185610ce2565b73ffffffffffffffffffffffffffffffffffffffff16816103ac6060870187610dcd565b6040516103ba929190610e32565b60006040518083038185875af1925050503d80600081146103f7576040519150601f19603f3d011682016040523d82523d6000602084013e6103fc565b606091505b50602080850191909152901515808452908501351761046d577f08c379a000000000000000000000000000000000000000000000000000000000600052602060045260176024527f4d756c746963616c6c333a2063616c6c206661696c656400000000000000000060445260846000fd5b5050600101610325565b508234146104e6576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601a60248201527f4d756c746963616c6c333a2076616c7565206d69736d6174636800000000000060448201526064015b60405180910390fd5b50505092915050565b436060828067ffffffffffffffff81111561050c5761050c610d31565b60405190808252806020026020018201604052801561053f57816020015b606081526020019060019003908161052a5790505b5091503660005b8281101561068657600087878381811061056257610562610d60565b90506020028101906105749190610e42565b92506105836020840184610ce2565b73ffffffffffffffffffffffffffffffffffffffff166105a66020850185610dcd565b6040516105b4929190610e32565b6000604051808303816000865af19150503d80600081146105f1576040519150601f19603f3d011682016040523d82523d6000602084013e6105f6565b606091505b5086848151811061060957610609610d60565b602090810291909101015290508061067d576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601760248201527f4d756c746963616c6c333a2063616c6c206661696c656400000000000000000060448201526064016104dd565b50600101610546565b5050509250929050565b43804060606106a086868661085a565b905093509350939050565b6060818067ffffffffffffffff8111156106c7576106c7610d31565b60405190808252806020026020018201604052801561070d57816020015b6040805180820190915260008152606060208201528152602001906001900390816106e55790505b5091503660005b828110156104e657600084828151811061073057610730610d60565b6020026020010151905086868381811061074c5761074c610d60565b905060200281019061075e9190610e76565b925061076d6020840184610ce2565b73ffffffffffffffffffffffffffffffffffffffff166107906040850185610dcd565b60405161079e929190610e32565b6000604051808303816000865af19150503d80600081146107db576040519150601f19603f3d011682016040523d82523d6000602084013e6107e0565b606091505b506020808401919091529015158083529084013517610851577f08c379a000000000000000000000000000000000000000000000000000000000600052602060045260176024527f4d756c746963616c6c333a2063616c6c206661696c656400000000000000000060445260646000fd5b50600101610714565b6060818067ffffffffffffffff81111561087657610876610d31565b6040519080825280602002602001820160405280156108bc57816020015b6040805180820190915260008152606060208201528152602001906001900390816108945790505b5091503660005b82811015610a105760008482815181106108df576108df610d60565b602002602001015190508686838181106108fb576108fb610d60565b905060200281019061090d9190610e42565b925061091c6020840184610ce2565b73ffffffffffffffffffffffffffffffffffffffff1661093f6020850185610dcd565b60405161094d929190610e32565b6000604051808303816000865af19150503d806000811461098a576040519150601f19603f3d011682016040523d82523d6000602084013e61098f565b606091505b506020830152151581528715610a07578051610a07576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601760248201527f4d756c746963616c6c333a2063616c6c206661696c656400000000000000000060448201526064016104dd565b506001016108c3565b5050509392505050565b6000806060610a2b60018686610690565b919790965090945092505050565b60008083601f840112610a4b57600080fd5b50813567ffffffffffffffff811115610a6357600080fd5b6020830191508360208260051b8501011115610a7e57600080fd5b9250929050565b60008060208385031215610a9857600080fd5b823567ffffffffffffffff811115610aaf57600080fd5b610abb85828601610a39565b90969095509350505050565b6000815180845260005b81811015610aed57602081850181015186830182015201610ad1565b81811115610aff576000602083870101525b50601f017fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe0169290920160200192915050565b600082825180855260208086019550808260051b84010181860160005b84811015610bb1578583037fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe001895281518051151584528401516040858501819052610b9d81860183610ac7565b9a86019a9450505090830190600101610b4f565b5090979650505050505050565b602081526000610bd16020830184610b32565b9392505050565b600060408201848352602060408185015281855180845260608601915060608160051b870101935082870160005b82811015610c52577fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffa0888703018452610c40868351610ac7565b95509284019290840190600101610c06565b509398975050505050505050565b600080600060408486031215610c7557600080fd5b83358015158114610c8557600080fd5b9250602084013567ffffffffffffffff811115610ca157600080fd5b610cad86828701610a39565b9497909650939450505050565b838152826020820152606060408201526000610cd96060830184610b32565b95945050505050565b600060208284031215610cf457600080fd5b813573ffffffffffffffffffffffffffffffffffffffff81168114610bd157600080fd5b600060208284031215610d2a57600080fd5b5035919050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052604160045260246000fd5b7f4e487b7100000000000000000000000000000000000000000000000000000000600052603260045260246000fd5b600082357fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff81833603018112610dc357600080fd5b9190910192915050565b60008083357fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe1843603018112610e0257600080fd5b83018035915067ffffffffffffffff821115610e1d57600080fd5b602001915036819003821315610a7e57600080fd5b8183823760009101908152919050565b600082357fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffc1833603018112610dc357600080fd5b600082357fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffa1833603018112610dc357600080fdfea2646970667358221220bb2b5c71a328032f97c676ae39a1ec2148d3e5d6f73d95e9b17910152d61f16264736f6c634300080c00331ca0edce47092c0f398cebf3ffc267f05c8e7076e3b89445e0fe50f6332273d4569ba01b0b9d000e19b24c5869b0fc3b22b0d6fa47cd63316875cbbd577d76e6fde086"; \
	fi

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
	$(MAKE) anvil-deploy-multicall3 network=anvil-localhost environment-type=local

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

# E.g. make configure-safe-module network=anvil-localhost environment-type=local node_address="0xcb192a4ba9abea8579c59231b8a57e3f0591e02b" safe_address=0x3A0A80BD107A4E029154286293296e62d235a4ce module_address=0xcE594ED193F314d66bC3Af9b53930CD9Ea87cc6E
.PHONY: configure-safe-module
configure-safe-module: ensure-environment-and-network-are-set ensure-privatekey-is-set
configure-safe-module: ## Configure existing node, safe and module
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
		--broadcast --sig "configureSafeModule(address[],address,address)" [$(node_address)] $(safe_address) $(module_address)

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

# E.g. make move-nodes-to-safe-module network=anvil-localhost environment-type=local node_addresses="0xc614b16027fd80c2b45ec51692260667157fd06a,0x02262ecbe2812f3138ce951c1898bc39f9868a18" safe_address=0xd46bd490944891D09c9a547432D140421C6Fdc48 module_address=0x9b2E3F03DD82516C8989d24dc2bDeA5C8C11D3a2
.PHONY: move-nodes-to-safe-module
move-nodes-to-safe-module: ensure-environment-and-network-are-set ensure-privatekey-is-set
move-nodes-to-safe-module: ## Move nodes attached to safe and module to a new safe module
ifeq ($(node_addresses),)
	echo "parameter <node_addresses> missing" >&2 && exit 1
endif
ifeq ($(safe_address),)
	echo "parameter <safe_address> missing" >&2 && exit 1
endif
ifeq ($(module_address),)
	echo "parameter <module_address> missing" >&2 && exit 1
endif
	$(forge-script) script/SingleAction.s.sol:SingleActionFromPrivateKeyScript \
		--broadcast --sig "moveNodesToSafeModule(address[],address,address)" [$(node_addresses)] $(safe_address) $(module_address)

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

# E.g. make manager-retry-register-nodes network=anvil-localhost environment-type=local node_addresses="[0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266,0x70997970c51812dc3a010c7d01b50e0d17dc79c8]"
.PHONY: manager-retry-register-nodes
manager-retry-register-nodes: ensure-environment-and-network-are-set ensure-privatekey-is-set
manager-retry-register-nodes: ## owner deregister given nodes from network registry contract
ifeq ($(node_addresses),)
	echo "parameter <node_addresses> missing" >&2 && exit 1
endif
	$(forge-script) script/SingleAction.s.sol:SingleActionFromPrivateKeyScript \
		--broadcast --sig "managerRetryNetworkRegistry(address[])" \
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
