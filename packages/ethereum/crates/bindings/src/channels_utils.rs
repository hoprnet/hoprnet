pub use channels_utils::*;
#[allow(clippy::too_many_arguments, non_camel_case_types)]
pub mod channels_utils {
    #![allow(clippy::enum_variant_names)]
    #![allow(dead_code)]
    #![allow(clippy::type_complexity)]
    #![allow(unused_imports)]
    pub use super::super::shared_types::*;
    use ethers::contract::{
        builders::{ContractCall, Event},
        Contract, Lazy,
    };
    use ethers::core::{
        abi::{Abi, Detokenize, InvalidOutputType, Token, Tokenizable},
        types::*,
    };
    use ethers::providers::Middleware;
    #[doc = "ChannelsUtils was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    # [rustfmt :: skip] const __ABI : & str = "[{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"bytes\",\"name\":\"publicKey\",\"type\":\"bytes\",\"components\":[],\"indexed\":false},{\"internalType\":\"bytes\",\"name\":\"multiaddr\",\"type\":\"bytes\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"Announcement\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"source\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"destination\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"bytes32\",\"name\":\"newCommitment\",\"type\":\"bytes32\",\"components\":[],\"indexed\":false},{\"internalType\":\"uint256\",\"name\":\"ticketEpoch\",\"type\":\"uint256\",\"components\":[],\"indexed\":false},{\"internalType\":\"uint256\",\"name\":\"channelBalance\",\"type\":\"uint256\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"ChannelBumped\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"funder\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"source\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"destination\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"ChannelFunded\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"source\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"destination\",\"type\":\"address\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"ChannelOpened\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"source\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"destination\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"struct HoprChannels.Channel\",\"name\":\"newState\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint256\",\"name\":\"balance\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"commitment\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"ticketEpoch\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"ticketIndex\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"enum HoprChannels.ChannelStatus\",\"name\":\"status\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"channelEpoch\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint32\",\"name\":\"closureTime\",\"type\":\"uint32\",\"components\":[]}],\"indexed\":false}],\"type\":\"event\",\"name\":\"ChannelUpdated\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"source\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"destination\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"bytes32\",\"name\":\"nextCommitment\",\"type\":\"bytes32\",\"components\":[],\"indexed\":false},{\"internalType\":\"uint256\",\"name\":\"ticketEpoch\",\"type\":\"uint256\",\"components\":[],\"indexed\":false},{\"internalType\":\"uint256\",\"name\":\"ticketIndex\",\"type\":\"uint256\",\"components\":[],\"indexed\":false},{\"internalType\":\"bytes32\",\"name\":\"proofOfRelaySecret\",\"type\":\"bytes32\",\"components\":[],\"indexed\":false},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[],\"indexed\":false},{\"internalType\":\"uint256\",\"name\":\"winProb\",\"type\":\"uint256\",\"components\":[],\"indexed\":false},{\"internalType\":\"bytes\",\"name\":\"signature\",\"type\":\"bytes\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"TicketRedeemed\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"log\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"log_address\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"uint256[]\",\"name\":\"val\",\"type\":\"uint256[]\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"log_array\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"int256[]\",\"name\":\"val\",\"type\":\"int256[]\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"log_array\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address[]\",\"name\":\"val\",\"type\":\"address[]\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"log_array\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"bytes\",\"name\":\"\",\"type\":\"bytes\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"log_bytes\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"log_bytes32\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"int256\",\"name\":\"\",\"type\":\"int256\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"log_int\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"key\",\"type\":\"string\",\"components\":[],\"indexed\":false},{\"internalType\":\"address\",\"name\":\"val\",\"type\":\"address\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"log_named_address\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"key\",\"type\":\"string\",\"components\":[],\"indexed\":false},{\"internalType\":\"uint256[]\",\"name\":\"val\",\"type\":\"uint256[]\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"log_named_array\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"key\",\"type\":\"string\",\"components\":[],\"indexed\":false},{\"internalType\":\"int256[]\",\"name\":\"val\",\"type\":\"int256[]\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"log_named_array\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"key\",\"type\":\"string\",\"components\":[],\"indexed\":false},{\"internalType\":\"address[]\",\"name\":\"val\",\"type\":\"address[]\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"log_named_array\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"key\",\"type\":\"string\",\"components\":[],\"indexed\":false},{\"internalType\":\"bytes\",\"name\":\"val\",\"type\":\"bytes\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"log_named_bytes\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"key\",\"type\":\"string\",\"components\":[],\"indexed\":false},{\"internalType\":\"bytes32\",\"name\":\"val\",\"type\":\"bytes32\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"log_named_bytes32\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"key\",\"type\":\"string\",\"components\":[],\"indexed\":false},{\"internalType\":\"int256\",\"name\":\"val\",\"type\":\"int256\",\"components\":[],\"indexed\":false},{\"internalType\":\"uint256\",\"name\":\"decimals\",\"type\":\"uint256\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"log_named_decimal_int\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"key\",\"type\":\"string\",\"components\":[],\"indexed\":false},{\"internalType\":\"uint256\",\"name\":\"val\",\"type\":\"uint256\",\"components\":[],\"indexed\":false},{\"internalType\":\"uint256\",\"name\":\"decimals\",\"type\":\"uint256\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"log_named_decimal_uint\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"key\",\"type\":\"string\",\"components\":[],\"indexed\":false},{\"internalType\":\"int256\",\"name\":\"val\",\"type\":\"int256\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"log_named_int\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"key\",\"type\":\"string\",\"components\":[],\"indexed\":false},{\"internalType\":\"string\",\"name\":\"val\",\"type\":\"string\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"log_named_string\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"key\",\"type\":\"string\",\"components\":[],\"indexed\":false},{\"internalType\":\"uint256\",\"name\":\"val\",\"type\":\"uint256\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"log_named_uint\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"log_string\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"log_uint\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"bytes\",\"name\":\"\",\"type\":\"bytes\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"logs\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"IS_SCRIPT\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"IS_TEST\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"SECRET_0\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"SECRET_1\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"SECRET_2\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"struct HoprChannels.Channel\",\"name\":\"channel1\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint256\",\"name\":\"balance\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"commitment\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"ticketEpoch\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"ticketIndex\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"enum HoprChannels.ChannelStatus\",\"name\":\"status\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"channelEpoch\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint32\",\"name\":\"closureTime\",\"type\":\"uint32\",\"components\":[]}]},{\"internalType\":\"struct HoprChannels.Channel\",\"name\":\"channel2\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint256\",\"name\":\"balance\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"commitment\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"ticketEpoch\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"ticketIndex\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"enum HoprChannels.ChannelStatus\",\"name\":\"status\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"channelEpoch\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint32\",\"name\":\"closureTime\",\"type\":\"uint32\",\"components\":[]}]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"assertEqChannels\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"failed\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"contract HoprChannels\",\"name\":\"hoprChannels\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"channelId\",\"type\":\"bytes32\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"getChannelFromTuple\",\"outputs\":[{\"internalType\":\"struct HoprChannels.Channel\",\"name\":\"\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint256\",\"name\":\"balance\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"commitment\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"ticketEpoch\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"ticketIndex\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"enum HoprChannels.ChannelStatus\",\"name\":\"status\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"channelEpoch\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint32\",\"name\":\"closureTime\",\"type\":\"uint32\",\"components\":[]}]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"source\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"destination\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"pure\",\"type\":\"function\",\"name\":\"getChannelId\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"vm\",\"outputs\":[{\"internalType\":\"contract Vm\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]}]" ;
    #[doc = r" The parsed JSON-ABI of the contract."]
    pub static CHANNELSUTILS_ABI: ethers::contract::Lazy<ethers::core::abi::Abi> =
        ethers::contract::Lazy::new(|| {
            ethers::core::utils::__serde_json::from_str(__ABI).expect("invalid abi")
        });
    #[doc = r" Bytecode of the #name contract"]
    pub static CHANNELSUTILS_BYTECODE: ethers::contract::Lazy<ethers::core::types::Bytes> =
        ethers::contract::Lazy::new(|| {
            "0x6000805462ff00ff191662010001179055651cd958dc995d60d21b60a05260066080527f65462b0520ef7d3df61b9992ed3bea0c56ead753be7c8b3614e0ce01e4cac41b600881905560c652602060a68190527f2ed293efa90dd457a302c46949588d43099e0897bccde03115db594d0ba6244560098190556101065260e6526101266040527f1bc505fad5af8fcddbd5a461de8aebafb0526fd1628b1a057e41b7123d53e190600a553480156100b557600080fd5b50610a71806100c56000396000f3fe608060405234801561001057600080fd5b506004361061009e5760003560e01c8063b013655a11610066578063b013655a14610176578063ba414fa614610196578063cfffcb8c1461019e578063f8ccbf47146101a7578063fa7626d4146101ba57600080fd5b80633a768463146100a35780635e939255146100db5780636901e571146100f25780637aa56adc146100fb5780637b0aed3d1461011e575b600080fd5b6100be737109709ecfa91a80626ff3989d68f67f5b1dd12d81565b6040516001600160a01b0390911681526020015b60405180910390f35b6100e460095481565b6040519081526020016100d2565b6100e4600a5481565b61010e61010936600461079d565b6101c7565b60405190151581526020016100d2565b6100e461012c3660046107e8565b6040516bffffffffffffffffffffffff19606084811b8216602084015283901b16603482015260009060480160405160208183030381529060405280519060200120905092915050565b610189610184366004610821565b6101f4565b6040516100d29190610863565b61010e61030e565b6100e460085481565b60005461010e9062010000900460ff1681565b60005461010e9060ff1681565b6000806101d384610439565b905060006101e084610439565b90506101ec828261048b565b505092915050565b6040805160e081018252600080825260208201819052918101829052606081018290526080810182905260a0810182905260c08101919091526000806000806000806000896001600160a01b0316637a7ebd7b8a6040518263ffffffff1660e01b815260040161026691815260200190565b60e060405180830381865afa158015610283573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906102a791906108d7565b96509650965096509650965096506040518060e001604052808881526020018781526020018681526020018581526020018460038111156102ea576102ea61084d565b81526020018381526020018263ffffffff1681525097505050505050505092915050565b60008054610100900460ff161561032e5750600054610100900460ff1690565b6000737109709ecfa91a80626ff3989d68f67f5b1dd12d3b156104345760408051737109709ecfa91a80626ff3989d68f67f5b1dd12d602082018190526519985a5b195960d21b828401528251808303840181526060830190935260009290916103bc917f667f9d70ca411d70ead50d8d5c22070dafc36ad75f3dcf5e7237b22ade9aecc49160800161097c565b60408051601f19818403018152908290526103d6916109a0565b6000604051808303816000865af19150503d8060008114610413576040519150601f19603f3d011682016040523d82523d6000602084013e610418565b606091505b509150508080602001905181019061043091906109b3565b9150505b919050565b80516020808301516040808501516060860151608087015160a088015160c0890151945160009861046e9890979691016109d5565b604051602081830303815290604052805190602001209050919050565b8082146105b5577f41304facd9323d75b11bcdd609cb38effffdb05710f7caf0e9b16c6d9d709f506040516104ff9060208082526025908201527f4572726f723a2061203d3d2062206e6f7420736174697366696564205b627974604082015264657333325d60d81b606082015260800190565b60405180910390a160408051818152600a81830152690808115e1c1958dd195960b21b60608201526020810183905290517fafb795c9c61e4fe7468c386f925d7a5429ecad9c0495ddb8d38d690614d32f999181900360800190a160408051818152600a8183015269080808081058dd1d585b60b21b60608201526020810184905290517fafb795c9c61e4fe7468c386f925d7a5429ecad9c0495ddb8d38d690614d32f999181900360800190a16105b56105b9565b5050565b737109709ecfa91a80626ff3989d68f67f5b1dd12d3b156106b45760408051737109709ecfa91a80626ff3989d68f67f5b1dd12d602082018190526519985a5b195960d21b9282019290925260016060820152600091907f70ca10bbd0dbfd9020a9f4b13402c16cb120705e0d1c0aeab10fa353ae586fc49060800160408051601f1981840301815290829052610653929160200161097c565b60408051601f198184030181529082905261066d916109a0565b6000604051808303816000865af19150503d80600081146106aa576040519150601f19603f3d011682016040523d82523d6000602084013e6106af565b606091505b505050505b6000805461ff001916610100179055565b600481106106d257600080fd5b50565b8035610434816106c5565b63ffffffff811681146106d257600080fd5b8035610434816106e0565b600060e0828403121561070f57600080fd5b60405160e0810181811067ffffffffffffffff8211171561074057634e487b7160e01b600052604160045260246000fd5b806040525080915082358152602083013560208201526040830135604082015260608301356060820152610776608084016106d5565b608082015260a083013560a082015261079160c084016106f2565b60c08201525092915050565b6000806101c083850312156107b157600080fd5b6107bb84846106fd565b91506107ca8460e085016106fd565b90509250929050565b6001600160a01b03811681146106d257600080fd5b600080604083850312156107fb57600080fd5b8235610806816107d3565b91506020830135610816816107d3565b809150509250929050565b6000806040838503121561083457600080fd5b823561083f816107d3565b946020939093013593505050565b634e487b7160e01b600052602160045260246000fd5b600060e082019050825182526020830151602083015260408301516040830152606083015160608301526080830151600481106108b057634e487b7160e01b600052602160045260246000fd5b8060808401525060a083015160a083015263ffffffff60c08401511660c083015292915050565b600080600080600080600060e0888a0312156108f257600080fd5b875196506020880151955060408801519450606088015193506080880151610919816106c5565b60a089015160c08a01519194509250610931816106e0565b8091505092959891949750929550565b6000815160005b818110156109625760208185018101518683015201610948565b81811115610971576000828601525b509290920192915050565b6001600160e01b03198316815260006109986004830184610941565b949350505050565b60006109ac8284610941565b9392505050565b6000602082840312156109c557600080fd5b815180151581146109ac57600080fd5b878152866020820152856040820152846060820152600060048510610a0a57634e487b7160e01b600052602160045260246000fd5b5060f89390931b6080840152608183019190915260e01b6001600160e01b03191660a182015260a50194935050505056fea264697066735822122000d18228d983d1dc3317aa708209627bea54c391d23fa97cda0a579dfde361f764736f6c634300080d0033" . parse () . expect ("invalid bytecode")
        });
    pub struct ChannelsUtils<M>(ethers::contract::Contract<M>);
    impl<M> Clone for ChannelsUtils<M> {
        fn clone(&self) -> Self {
            ChannelsUtils(self.0.clone())
        }
    }
    impl<M> std::ops::Deref for ChannelsUtils<M> {
        type Target = ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> std::fmt::Debug for ChannelsUtils<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(ChannelsUtils))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ethers::providers::Middleware> ChannelsUtils<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            ethers::contract::Contract::new(address.into(), CHANNELSUTILS_ABI.clone(), client)
                .into()
        }
        #[doc = r" Constructs the general purpose `Deployer` instance based on the provided constructor arguments and sends it."]
        #[doc = r" Returns a new instance of a deployer that returns an instance of this contract after sending the transaction"]
        #[doc = r""]
        #[doc = r" Notes:"]
        #[doc = r" 1. If there are no constructor arguments, you should pass `()` as the argument."]
        #[doc = r" 1. The default poll duration is 7 seconds."]
        #[doc = r" 1. The default number of confirmations is 1 block."]
        #[doc = r""]
        #[doc = r""]
        #[doc = r" # Example"]
        #[doc = r""]
        #[doc = r" Generate contract bindings with `abigen!` and deploy a new contract instance."]
        #[doc = r""]
        #[doc = r" *Note*: this requires a `bytecode` and `abi` object in the `greeter.json` artifact."]
        #[doc = r""]
        #[doc = r" ```ignore"]
        #[doc = r" # async fn deploy<M: ethers::providers::Middleware>(client: ::std::sync::Arc<M>) {"]
        #[doc = r#"     abigen!(Greeter,"../greeter.json");"#]
        #[doc = r""]
        #[doc = r#"    let greeter_contract = Greeter::deploy(client, "Hello world!".to_string()).unwrap().send().await.unwrap();"#]
        #[doc = r"    let msg = greeter_contract.greet().call().await.unwrap();"]
        #[doc = r" # }"]
        #[doc = r" ```"]
        pub fn deploy<T: ethers::core::abi::Tokenize>(
            client: ::std::sync::Arc<M>,
            constructor_args: T,
        ) -> ::std::result::Result<
            ethers::contract::builders::ContractDeployer<M, Self>,
            ethers::contract::ContractError<M>,
        > {
            let factory = ethers::contract::ContractFactory::new(
                CHANNELSUTILS_ABI.clone(),
                CHANNELSUTILS_BYTECODE.clone().into(),
                client,
            );
            let deployer = factory.deploy(constructor_args)?;
            let deployer = ethers::contract::ContractDeployer::new(deployer);
            Ok(deployer)
        }
        #[doc = "Calls the contract's `IS_SCRIPT` (0xf8ccbf47) function"]
        pub fn is_script(&self) -> ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([248, 204, 191, 71], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `IS_TEST` (0xfa7626d4) function"]
        pub fn is_test(&self) -> ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([250, 118, 38, 212], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `SECRET_0` (0xcfffcb8c) function"]
        pub fn secret_0(&self) -> ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([207, 255, 203, 140], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `SECRET_1` (0x5e939255) function"]
        pub fn secret_1(&self) -> ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([94, 147, 146, 85], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `SECRET_2` (0x6901e571) function"]
        pub fn secret_2(&self) -> ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([105, 1, 229, 113], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `assertEqChannels` (0x7aa56adc) function"]
        pub fn assert_eq_channels(
            &self,
            channel_1: Channel,
            channel_2: Channel,
        ) -> ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([122, 165, 106, 220], (channel_1, channel_2))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `failed` (0xba414fa6) function"]
        pub fn failed(&self) -> ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([186, 65, 79, 166], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `getChannelFromTuple` (0xb013655a) function"]
        pub fn get_channel_from_tuple(
            &self,
            hopr_channels: ethers::core::types::Address,
            channel_id: [u8; 32],
        ) -> ethers::contract::builders::ContractCall<M, Channel> {
            self.0
                .method_hash([176, 19, 101, 90], (hopr_channels, channel_id))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `getChannelId` (0x7b0aed3d) function"]
        pub fn get_channel_id(
            &self,
            source: ethers::core::types::Address,
            destination: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([123, 10, 237, 61], (source, destination))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `vm` (0x3a768463) function"]
        pub fn vm(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Address> {
            self.0
                .method_hash([58, 118, 132, 99], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Gets the contract's `Announcement` event"]
        pub fn announcement_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, AnnouncementFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `ChannelBumped` event"]
        pub fn channel_bumped_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, ChannelBumpedFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `ChannelFunded` event"]
        pub fn channel_funded_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, ChannelFundedFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `ChannelOpened` event"]
        pub fn channel_opened_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, ChannelOpenedFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `ChannelUpdated` event"]
        pub fn channel_updated_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, ChannelUpdatedFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `TicketRedeemed` event"]
        pub fn ticket_redeemed_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, TicketRedeemedFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `log` event"]
        pub fn log_filter(&self) -> ethers::contract::builders::Event<M, LogFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `log_address` event"]
        pub fn log_address_filter(&self) -> ethers::contract::builders::Event<M, LogAddressFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `log_array` event"]
        pub fn log_array_1_filter(&self) -> ethers::contract::builders::Event<M, LogArray1Filter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `log_array` event"]
        pub fn log_array_2_filter(&self) -> ethers::contract::builders::Event<M, LogArray2Filter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `log_array` event"]
        pub fn log_array_3_filter(&self) -> ethers::contract::builders::Event<M, LogArray3Filter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `log_bytes` event"]
        pub fn log_bytes_filter(&self) -> ethers::contract::builders::Event<M, LogBytesFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `log_bytes32` event"]
        pub fn log_bytes_32_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, LogBytes32Filter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `log_int` event"]
        pub fn log_int_filter(&self) -> ethers::contract::builders::Event<M, LogIntFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `log_named_address` event"]
        pub fn log_named_address_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, LogNamedAddressFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `log_named_array` event"]
        pub fn log_named_array_1_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, LogNamedArray1Filter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `log_named_array` event"]
        pub fn log_named_array_2_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, LogNamedArray2Filter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `log_named_array` event"]
        pub fn log_named_array_3_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, LogNamedArray3Filter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `log_named_bytes` event"]
        pub fn log_named_bytes_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, LogNamedBytesFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `log_named_bytes32` event"]
        pub fn log_named_bytes_32_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, LogNamedBytes32Filter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `log_named_decimal_int` event"]
        pub fn log_named_decimal_int_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, LogNamedDecimalIntFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `log_named_decimal_uint` event"]
        pub fn log_named_decimal_uint_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, LogNamedDecimalUintFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `log_named_int` event"]
        pub fn log_named_int_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, LogNamedIntFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `log_named_string` event"]
        pub fn log_named_string_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, LogNamedStringFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `log_named_uint` event"]
        pub fn log_named_uint_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, LogNamedUintFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `log_string` event"]
        pub fn log_string_filter(&self) -> ethers::contract::builders::Event<M, LogStringFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `log_uint` event"]
        pub fn log_uint_filter(&self) -> ethers::contract::builders::Event<M, LogUintFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `logs` event"]
        pub fn logs_filter(&self) -> ethers::contract::builders::Event<M, LogsFilter> {
            self.0.event()
        }
        #[doc = r" Returns an [`Event`](#ethers_contract::builders::Event) builder for all events of this contract"]
        pub fn events(&self) -> ethers::contract::builders::Event<M, ChannelsUtilsEvents> {
            self.0.event_with_filter(Default::default())
        }
    }
    impl<M: ethers::providers::Middleware> From<ethers::contract::Contract<M>> for ChannelsUtils<M> {
        fn from(contract: ethers::contract::Contract<M>) -> Self {
            Self(contract)
        }
    }
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthEvent,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethevent(name = "Announcement", abi = "Announcement(address,bytes,bytes)")]
    pub struct AnnouncementFilter {
        #[ethevent(indexed)]
        pub account: ethers::core::types::Address,
        pub public_key: ethers::core::types::Bytes,
        pub multiaddr: ethers::core::types::Bytes,
    }
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthEvent,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethevent(
        name = "ChannelBumped",
        abi = "ChannelBumped(address,address,bytes32,uint256,uint256)"
    )]
    pub struct ChannelBumpedFilter {
        #[ethevent(indexed)]
        pub source: ethers::core::types::Address,
        #[ethevent(indexed)]
        pub destination: ethers::core::types::Address,
        pub new_commitment: [u8; 32],
        pub ticket_epoch: ethers::core::types::U256,
        pub channel_balance: ethers::core::types::U256,
    }
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthEvent,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethevent(
        name = "ChannelFunded",
        abi = "ChannelFunded(address,address,address,uint256)"
    )]
    pub struct ChannelFundedFilter {
        #[ethevent(indexed)]
        pub funder: ethers::core::types::Address,
        #[ethevent(indexed)]
        pub source: ethers::core::types::Address,
        #[ethevent(indexed)]
        pub destination: ethers::core::types::Address,
        pub amount: ethers::core::types::U256,
    }
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthEvent,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethevent(name = "ChannelOpened", abi = "ChannelOpened(address,address)")]
    pub struct ChannelOpenedFilter {
        #[ethevent(indexed)]
        pub source: ethers::core::types::Address,
        #[ethevent(indexed)]
        pub destination: ethers::core::types::Address,
    }
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthEvent,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethevent(
        name = "ChannelUpdated",
        abi = "ChannelUpdated(address,address,(uint256,bytes32,uint256,uint256,uint8,uint256,uint32))"
    )]
    pub struct ChannelUpdatedFilter {
        #[ethevent(indexed)]
        pub source: ethers::core::types::Address,
        #[ethevent(indexed)]
        pub destination: ethers::core::types::Address,
        pub new_state: Channel,
    }
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthEvent,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethevent(
        name = "TicketRedeemed",
        abi = "TicketRedeemed(address,address,bytes32,uint256,uint256,bytes32,uint256,uint256,bytes)"
    )]
    pub struct TicketRedeemedFilter {
        #[ethevent(indexed)]
        pub source: ethers::core::types::Address,
        #[ethevent(indexed)]
        pub destination: ethers::core::types::Address,
        pub next_commitment: [u8; 32],
        pub ticket_epoch: ethers::core::types::U256,
        pub ticket_index: ethers::core::types::U256,
        pub proof_of_relay_secret: [u8; 32],
        pub amount: ethers::core::types::U256,
        pub win_prob: ethers::core::types::U256,
        pub signature: ethers::core::types::Bytes,
    }
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthEvent,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethevent(name = "log", abi = "log(string)")]
    pub struct LogFilter(pub String);
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthEvent,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethevent(name = "log_address", abi = "log_address(address)")]
    pub struct LogAddressFilter(pub ethers::core::types::Address);
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthEvent,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethevent(name = "log_array", abi = "log_array(uint256[])")]
    pub struct LogArray1Filter {
        pub val: Vec<ethers::core::types::U256>,
    }
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthEvent,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethevent(name = "log_array", abi = "log_array(int256[])")]
    pub struct LogArray2Filter {
        pub val: Vec<I256>,
    }
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthEvent,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethevent(name = "log_array", abi = "log_array(address[])")]
    pub struct LogArray3Filter {
        pub val: Vec<ethers::core::types::Address>,
    }
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthEvent,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethevent(name = "log_bytes", abi = "log_bytes(bytes)")]
    pub struct LogBytesFilter(pub ethers::core::types::Bytes);
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthEvent,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethevent(name = "log_bytes32", abi = "log_bytes32(bytes32)")]
    pub struct LogBytes32Filter(pub [u8; 32]);
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthEvent,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethevent(name = "log_int", abi = "log_int(int256)")]
    pub struct LogIntFilter(pub I256);
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthEvent,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethevent(name = "log_named_address", abi = "log_named_address(string,address)")]
    pub struct LogNamedAddressFilter {
        pub key: String,
        pub val: ethers::core::types::Address,
    }
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthEvent,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethevent(name = "log_named_array", abi = "log_named_array(string,uint256[])")]
    pub struct LogNamedArray1Filter {
        pub key: String,
        pub val: Vec<ethers::core::types::U256>,
    }
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthEvent,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethevent(name = "log_named_array", abi = "log_named_array(string,int256[])")]
    pub struct LogNamedArray2Filter {
        pub key: String,
        pub val: Vec<I256>,
    }
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthEvent,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethevent(name = "log_named_array", abi = "log_named_array(string,address[])")]
    pub struct LogNamedArray3Filter {
        pub key: String,
        pub val: Vec<ethers::core::types::Address>,
    }
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthEvent,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethevent(name = "log_named_bytes", abi = "log_named_bytes(string,bytes)")]
    pub struct LogNamedBytesFilter {
        pub key: String,
        pub val: ethers::core::types::Bytes,
    }
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthEvent,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethevent(name = "log_named_bytes32", abi = "log_named_bytes32(string,bytes32)")]
    pub struct LogNamedBytes32Filter {
        pub key: String,
        pub val: [u8; 32],
    }
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthEvent,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethevent(
        name = "log_named_decimal_int",
        abi = "log_named_decimal_int(string,int256,uint256)"
    )]
    pub struct LogNamedDecimalIntFilter {
        pub key: String,
        pub val: I256,
        pub decimals: ethers::core::types::U256,
    }
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthEvent,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethevent(
        name = "log_named_decimal_uint",
        abi = "log_named_decimal_uint(string,uint256,uint256)"
    )]
    pub struct LogNamedDecimalUintFilter {
        pub key: String,
        pub val: ethers::core::types::U256,
        pub decimals: ethers::core::types::U256,
    }
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthEvent,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethevent(name = "log_named_int", abi = "log_named_int(string,int256)")]
    pub struct LogNamedIntFilter {
        pub key: String,
        pub val: I256,
    }
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthEvent,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethevent(name = "log_named_string", abi = "log_named_string(string,string)")]
    pub struct LogNamedStringFilter {
        pub key: String,
        pub val: String,
    }
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthEvent,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethevent(name = "log_named_uint", abi = "log_named_uint(string,uint256)")]
    pub struct LogNamedUintFilter {
        pub key: String,
        pub val: ethers::core::types::U256,
    }
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthEvent,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethevent(name = "log_string", abi = "log_string(string)")]
    pub struct LogStringFilter(pub String);
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthEvent,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethevent(name = "log_uint", abi = "log_uint(uint256)")]
    pub struct LogUintFilter(pub ethers::core::types::U256);
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthEvent,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethevent(name = "logs", abi = "logs(bytes)")]
    pub struct LogsFilter(pub ethers::core::types::Bytes);
    #[derive(Debug, Clone, PartialEq, Eq, ethers :: contract :: EthAbiType)]
    pub enum ChannelsUtilsEvents {
        AnnouncementFilter(AnnouncementFilter),
        ChannelBumpedFilter(ChannelBumpedFilter),
        ChannelFundedFilter(ChannelFundedFilter),
        ChannelOpenedFilter(ChannelOpenedFilter),
        ChannelUpdatedFilter(ChannelUpdatedFilter),
        TicketRedeemedFilter(TicketRedeemedFilter),
        LogFilter(LogFilter),
        LogAddressFilter(LogAddressFilter),
        LogArray1Filter(LogArray1Filter),
        LogArray2Filter(LogArray2Filter),
        LogArray3Filter(LogArray3Filter),
        LogBytesFilter(LogBytesFilter),
        LogBytes32Filter(LogBytes32Filter),
        LogIntFilter(LogIntFilter),
        LogNamedAddressFilter(LogNamedAddressFilter),
        LogNamedArray1Filter(LogNamedArray1Filter),
        LogNamedArray2Filter(LogNamedArray2Filter),
        LogNamedArray3Filter(LogNamedArray3Filter),
        LogNamedBytesFilter(LogNamedBytesFilter),
        LogNamedBytes32Filter(LogNamedBytes32Filter),
        LogNamedDecimalIntFilter(LogNamedDecimalIntFilter),
        LogNamedDecimalUintFilter(LogNamedDecimalUintFilter),
        LogNamedIntFilter(LogNamedIntFilter),
        LogNamedStringFilter(LogNamedStringFilter),
        LogNamedUintFilter(LogNamedUintFilter),
        LogStringFilter(LogStringFilter),
        LogUintFilter(LogUintFilter),
        LogsFilter(LogsFilter),
    }
    impl ethers::contract::EthLogDecode for ChannelsUtilsEvents {
        fn decode_log(
            log: &ethers::core::abi::RawLog,
        ) -> ::std::result::Result<Self, ethers::core::abi::Error>
        where
            Self: Sized,
        {
            if let Ok(decoded) = AnnouncementFilter::decode_log(log) {
                return Ok(ChannelsUtilsEvents::AnnouncementFilter(decoded));
            }
            if let Ok(decoded) = ChannelBumpedFilter::decode_log(log) {
                return Ok(ChannelsUtilsEvents::ChannelBumpedFilter(decoded));
            }
            if let Ok(decoded) = ChannelFundedFilter::decode_log(log) {
                return Ok(ChannelsUtilsEvents::ChannelFundedFilter(decoded));
            }
            if let Ok(decoded) = ChannelOpenedFilter::decode_log(log) {
                return Ok(ChannelsUtilsEvents::ChannelOpenedFilter(decoded));
            }
            if let Ok(decoded) = ChannelUpdatedFilter::decode_log(log) {
                return Ok(ChannelsUtilsEvents::ChannelUpdatedFilter(decoded));
            }
            if let Ok(decoded) = TicketRedeemedFilter::decode_log(log) {
                return Ok(ChannelsUtilsEvents::TicketRedeemedFilter(decoded));
            }
            if let Ok(decoded) = LogFilter::decode_log(log) {
                return Ok(ChannelsUtilsEvents::LogFilter(decoded));
            }
            if let Ok(decoded) = LogAddressFilter::decode_log(log) {
                return Ok(ChannelsUtilsEvents::LogAddressFilter(decoded));
            }
            if let Ok(decoded) = LogArray1Filter::decode_log(log) {
                return Ok(ChannelsUtilsEvents::LogArray1Filter(decoded));
            }
            if let Ok(decoded) = LogArray2Filter::decode_log(log) {
                return Ok(ChannelsUtilsEvents::LogArray2Filter(decoded));
            }
            if let Ok(decoded) = LogArray3Filter::decode_log(log) {
                return Ok(ChannelsUtilsEvents::LogArray3Filter(decoded));
            }
            if let Ok(decoded) = LogBytesFilter::decode_log(log) {
                return Ok(ChannelsUtilsEvents::LogBytesFilter(decoded));
            }
            if let Ok(decoded) = LogBytes32Filter::decode_log(log) {
                return Ok(ChannelsUtilsEvents::LogBytes32Filter(decoded));
            }
            if let Ok(decoded) = LogIntFilter::decode_log(log) {
                return Ok(ChannelsUtilsEvents::LogIntFilter(decoded));
            }
            if let Ok(decoded) = LogNamedAddressFilter::decode_log(log) {
                return Ok(ChannelsUtilsEvents::LogNamedAddressFilter(decoded));
            }
            if let Ok(decoded) = LogNamedArray1Filter::decode_log(log) {
                return Ok(ChannelsUtilsEvents::LogNamedArray1Filter(decoded));
            }
            if let Ok(decoded) = LogNamedArray2Filter::decode_log(log) {
                return Ok(ChannelsUtilsEvents::LogNamedArray2Filter(decoded));
            }
            if let Ok(decoded) = LogNamedArray3Filter::decode_log(log) {
                return Ok(ChannelsUtilsEvents::LogNamedArray3Filter(decoded));
            }
            if let Ok(decoded) = LogNamedBytesFilter::decode_log(log) {
                return Ok(ChannelsUtilsEvents::LogNamedBytesFilter(decoded));
            }
            if let Ok(decoded) = LogNamedBytes32Filter::decode_log(log) {
                return Ok(ChannelsUtilsEvents::LogNamedBytes32Filter(decoded));
            }
            if let Ok(decoded) = LogNamedDecimalIntFilter::decode_log(log) {
                return Ok(ChannelsUtilsEvents::LogNamedDecimalIntFilter(decoded));
            }
            if let Ok(decoded) = LogNamedDecimalUintFilter::decode_log(log) {
                return Ok(ChannelsUtilsEvents::LogNamedDecimalUintFilter(decoded));
            }
            if let Ok(decoded) = LogNamedIntFilter::decode_log(log) {
                return Ok(ChannelsUtilsEvents::LogNamedIntFilter(decoded));
            }
            if let Ok(decoded) = LogNamedStringFilter::decode_log(log) {
                return Ok(ChannelsUtilsEvents::LogNamedStringFilter(decoded));
            }
            if let Ok(decoded) = LogNamedUintFilter::decode_log(log) {
                return Ok(ChannelsUtilsEvents::LogNamedUintFilter(decoded));
            }
            if let Ok(decoded) = LogStringFilter::decode_log(log) {
                return Ok(ChannelsUtilsEvents::LogStringFilter(decoded));
            }
            if let Ok(decoded) = LogUintFilter::decode_log(log) {
                return Ok(ChannelsUtilsEvents::LogUintFilter(decoded));
            }
            if let Ok(decoded) = LogsFilter::decode_log(log) {
                return Ok(ChannelsUtilsEvents::LogsFilter(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData)
        }
    }
    impl ::std::fmt::Display for ChannelsUtilsEvents {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                ChannelsUtilsEvents::AnnouncementFilter(element) => element.fmt(f),
                ChannelsUtilsEvents::ChannelBumpedFilter(element) => element.fmt(f),
                ChannelsUtilsEvents::ChannelFundedFilter(element) => element.fmt(f),
                ChannelsUtilsEvents::ChannelOpenedFilter(element) => element.fmt(f),
                ChannelsUtilsEvents::ChannelUpdatedFilter(element) => element.fmt(f),
                ChannelsUtilsEvents::TicketRedeemedFilter(element) => element.fmt(f),
                ChannelsUtilsEvents::LogFilter(element) => element.fmt(f),
                ChannelsUtilsEvents::LogAddressFilter(element) => element.fmt(f),
                ChannelsUtilsEvents::LogArray1Filter(element) => element.fmt(f),
                ChannelsUtilsEvents::LogArray2Filter(element) => element.fmt(f),
                ChannelsUtilsEvents::LogArray3Filter(element) => element.fmt(f),
                ChannelsUtilsEvents::LogBytesFilter(element) => element.fmt(f),
                ChannelsUtilsEvents::LogBytes32Filter(element) => element.fmt(f),
                ChannelsUtilsEvents::LogIntFilter(element) => element.fmt(f),
                ChannelsUtilsEvents::LogNamedAddressFilter(element) => element.fmt(f),
                ChannelsUtilsEvents::LogNamedArray1Filter(element) => element.fmt(f),
                ChannelsUtilsEvents::LogNamedArray2Filter(element) => element.fmt(f),
                ChannelsUtilsEvents::LogNamedArray3Filter(element) => element.fmt(f),
                ChannelsUtilsEvents::LogNamedBytesFilter(element) => element.fmt(f),
                ChannelsUtilsEvents::LogNamedBytes32Filter(element) => element.fmt(f),
                ChannelsUtilsEvents::LogNamedDecimalIntFilter(element) => element.fmt(f),
                ChannelsUtilsEvents::LogNamedDecimalUintFilter(element) => element.fmt(f),
                ChannelsUtilsEvents::LogNamedIntFilter(element) => element.fmt(f),
                ChannelsUtilsEvents::LogNamedStringFilter(element) => element.fmt(f),
                ChannelsUtilsEvents::LogNamedUintFilter(element) => element.fmt(f),
                ChannelsUtilsEvents::LogStringFilter(element) => element.fmt(f),
                ChannelsUtilsEvents::LogUintFilter(element) => element.fmt(f),
                ChannelsUtilsEvents::LogsFilter(element) => element.fmt(f),
            }
        }
    }
    #[doc = "Container type for all input parameters for the `IS_SCRIPT` function with signature `IS_SCRIPT()` and selector `[248, 204, 191, 71]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "IS_SCRIPT", abi = "IS_SCRIPT()")]
    pub struct IsScriptCall;
    #[doc = "Container type for all input parameters for the `IS_TEST` function with signature `IS_TEST()` and selector `[250, 118, 38, 212]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "IS_TEST", abi = "IS_TEST()")]
    pub struct IsTestCall;
    #[doc = "Container type for all input parameters for the `SECRET_0` function with signature `SECRET_0()` and selector `[207, 255, 203, 140]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "SECRET_0", abi = "SECRET_0()")]
    pub struct Secret0Call;
    #[doc = "Container type for all input parameters for the `SECRET_1` function with signature `SECRET_1()` and selector `[94, 147, 146, 85]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "SECRET_1", abi = "SECRET_1()")]
    pub struct Secret1Call;
    #[doc = "Container type for all input parameters for the `SECRET_2` function with signature `SECRET_2()` and selector `[105, 1, 229, 113]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "SECRET_2", abi = "SECRET_2()")]
    pub struct Secret2Call;
    #[doc = "Container type for all input parameters for the `assertEqChannels` function with signature `assertEqChannels((uint256,bytes32,uint256,uint256,uint8,uint256,uint32),(uint256,bytes32,uint256,uint256,uint8,uint256,uint32))` and selector `[122, 165, 106, 220]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(
        name = "assertEqChannels",
        abi = "assertEqChannels((uint256,bytes32,uint256,uint256,uint8,uint256,uint32),(uint256,bytes32,uint256,uint256,uint8,uint256,uint32))"
    )]
    pub struct AssertEqChannelsCall {
        pub channel_1: Channel,
        pub channel_2: Channel,
    }
    #[doc = "Container type for all input parameters for the `failed` function with signature `failed()` and selector `[186, 65, 79, 166]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "failed", abi = "failed()")]
    pub struct FailedCall;
    #[doc = "Container type for all input parameters for the `getChannelFromTuple` function with signature `getChannelFromTuple(address,bytes32)` and selector `[176, 19, 101, 90]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(
        name = "getChannelFromTuple",
        abi = "getChannelFromTuple(address,bytes32)"
    )]
    pub struct GetChannelFromTupleCall {
        pub hopr_channels: ethers::core::types::Address,
        pub channel_id: [u8; 32],
    }
    #[doc = "Container type for all input parameters for the `getChannelId` function with signature `getChannelId(address,address)` and selector `[123, 10, 237, 61]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "getChannelId", abi = "getChannelId(address,address)")]
    pub struct GetChannelIdCall {
        pub source: ethers::core::types::Address,
        pub destination: ethers::core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `vm` function with signature `vm()` and selector `[58, 118, 132, 99]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "vm", abi = "vm()")]
    pub struct VmCall;
    #[derive(Debug, Clone, PartialEq, Eq, ethers :: contract :: EthAbiType)]
    pub enum ChannelsUtilsCalls {
        IsScript(IsScriptCall),
        IsTest(IsTestCall),
        Secret0(Secret0Call),
        Secret1(Secret1Call),
        Secret2(Secret2Call),
        AssertEqChannels(AssertEqChannelsCall),
        Failed(FailedCall),
        GetChannelFromTuple(GetChannelFromTupleCall),
        GetChannelId(GetChannelIdCall),
        Vm(VmCall),
    }
    impl ethers::core::abi::AbiDecode for ChannelsUtilsCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::std::result::Result<Self, ethers::core::abi::AbiError> {
            if let Ok(decoded) =
                <IsScriptCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ChannelsUtilsCalls::IsScript(decoded));
            }
            if let Ok(decoded) = <IsTestCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ChannelsUtilsCalls::IsTest(decoded));
            }
            if let Ok(decoded) =
                <Secret0Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ChannelsUtilsCalls::Secret0(decoded));
            }
            if let Ok(decoded) =
                <Secret1Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ChannelsUtilsCalls::Secret1(decoded));
            }
            if let Ok(decoded) =
                <Secret2Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ChannelsUtilsCalls::Secret2(decoded));
            }
            if let Ok(decoded) =
                <AssertEqChannelsCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ChannelsUtilsCalls::AssertEqChannels(decoded));
            }
            if let Ok(decoded) = <FailedCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ChannelsUtilsCalls::Failed(decoded));
            }
            if let Ok(decoded) =
                <GetChannelFromTupleCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ChannelsUtilsCalls::GetChannelFromTuple(decoded));
            }
            if let Ok(decoded) =
                <GetChannelIdCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ChannelsUtilsCalls::GetChannelId(decoded));
            }
            if let Ok(decoded) = <VmCall as ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(ChannelsUtilsCalls::Vm(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ethers::core::abi::AbiEncode for ChannelsUtilsCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                ChannelsUtilsCalls::IsScript(element) => element.encode(),
                ChannelsUtilsCalls::IsTest(element) => element.encode(),
                ChannelsUtilsCalls::Secret0(element) => element.encode(),
                ChannelsUtilsCalls::Secret1(element) => element.encode(),
                ChannelsUtilsCalls::Secret2(element) => element.encode(),
                ChannelsUtilsCalls::AssertEqChannels(element) => element.encode(),
                ChannelsUtilsCalls::Failed(element) => element.encode(),
                ChannelsUtilsCalls::GetChannelFromTuple(element) => element.encode(),
                ChannelsUtilsCalls::GetChannelId(element) => element.encode(),
                ChannelsUtilsCalls::Vm(element) => element.encode(),
            }
        }
    }
    impl ::std::fmt::Display for ChannelsUtilsCalls {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                ChannelsUtilsCalls::IsScript(element) => element.fmt(f),
                ChannelsUtilsCalls::IsTest(element) => element.fmt(f),
                ChannelsUtilsCalls::Secret0(element) => element.fmt(f),
                ChannelsUtilsCalls::Secret1(element) => element.fmt(f),
                ChannelsUtilsCalls::Secret2(element) => element.fmt(f),
                ChannelsUtilsCalls::AssertEqChannels(element) => element.fmt(f),
                ChannelsUtilsCalls::Failed(element) => element.fmt(f),
                ChannelsUtilsCalls::GetChannelFromTuple(element) => element.fmt(f),
                ChannelsUtilsCalls::GetChannelId(element) => element.fmt(f),
                ChannelsUtilsCalls::Vm(element) => element.fmt(f),
            }
        }
    }
    impl ::std::convert::From<IsScriptCall> for ChannelsUtilsCalls {
        fn from(var: IsScriptCall) -> Self {
            ChannelsUtilsCalls::IsScript(var)
        }
    }
    impl ::std::convert::From<IsTestCall> for ChannelsUtilsCalls {
        fn from(var: IsTestCall) -> Self {
            ChannelsUtilsCalls::IsTest(var)
        }
    }
    impl ::std::convert::From<Secret0Call> for ChannelsUtilsCalls {
        fn from(var: Secret0Call) -> Self {
            ChannelsUtilsCalls::Secret0(var)
        }
    }
    impl ::std::convert::From<Secret1Call> for ChannelsUtilsCalls {
        fn from(var: Secret1Call) -> Self {
            ChannelsUtilsCalls::Secret1(var)
        }
    }
    impl ::std::convert::From<Secret2Call> for ChannelsUtilsCalls {
        fn from(var: Secret2Call) -> Self {
            ChannelsUtilsCalls::Secret2(var)
        }
    }
    impl ::std::convert::From<AssertEqChannelsCall> for ChannelsUtilsCalls {
        fn from(var: AssertEqChannelsCall) -> Self {
            ChannelsUtilsCalls::AssertEqChannels(var)
        }
    }
    impl ::std::convert::From<FailedCall> for ChannelsUtilsCalls {
        fn from(var: FailedCall) -> Self {
            ChannelsUtilsCalls::Failed(var)
        }
    }
    impl ::std::convert::From<GetChannelFromTupleCall> for ChannelsUtilsCalls {
        fn from(var: GetChannelFromTupleCall) -> Self {
            ChannelsUtilsCalls::GetChannelFromTuple(var)
        }
    }
    impl ::std::convert::From<GetChannelIdCall> for ChannelsUtilsCalls {
        fn from(var: GetChannelIdCall) -> Self {
            ChannelsUtilsCalls::GetChannelId(var)
        }
    }
    impl ::std::convert::From<VmCall> for ChannelsUtilsCalls {
        fn from(var: VmCall) -> Self {
            ChannelsUtilsCalls::Vm(var)
        }
    }
    #[doc = "Container type for all return fields from the `IS_SCRIPT` function with signature `IS_SCRIPT()` and selector `[248, 204, 191, 71]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct IsScriptReturn(pub bool);
    #[doc = "Container type for all return fields from the `IS_TEST` function with signature `IS_TEST()` and selector `[250, 118, 38, 212]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct IsTestReturn(pub bool);
    #[doc = "Container type for all return fields from the `SECRET_0` function with signature `SECRET_0()` and selector `[207, 255, 203, 140]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct Secret0Return(pub [u8; 32]);
    #[doc = "Container type for all return fields from the `SECRET_1` function with signature `SECRET_1()` and selector `[94, 147, 146, 85]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct Secret1Return(pub [u8; 32]);
    #[doc = "Container type for all return fields from the `SECRET_2` function with signature `SECRET_2()` and selector `[105, 1, 229, 113]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct Secret2Return(pub [u8; 32]);
    #[doc = "Container type for all return fields from the `assertEqChannels` function with signature `assertEqChannels((uint256,bytes32,uint256,uint256,uint8,uint256,uint32),(uint256,bytes32,uint256,uint256,uint8,uint256,uint32))` and selector `[122, 165, 106, 220]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct AssertEqChannelsReturn(pub bool);
    #[doc = "Container type for all return fields from the `failed` function with signature `failed()` and selector `[186, 65, 79, 166]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct FailedReturn(pub bool);
    #[doc = "Container type for all return fields from the `getChannelFromTuple` function with signature `getChannelFromTuple(address,bytes32)` and selector `[176, 19, 101, 90]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct GetChannelFromTupleReturn(pub Channel);
    #[doc = "Container type for all return fields from the `getChannelId` function with signature `getChannelId(address,address)` and selector `[123, 10, 237, 61]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct GetChannelIdReturn(pub [u8; 32]);
    #[doc = "Container type for all return fields from the `vm` function with signature `vm()` and selector `[58, 118, 132, 99]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct VmReturn(pub ethers::core::types::Address);
}
