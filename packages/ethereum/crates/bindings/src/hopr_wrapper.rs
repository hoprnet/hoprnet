pub use hopr_wrapper::*;
#[allow(clippy::too_many_arguments, non_camel_case_types)]
pub mod hopr_wrapper {
    #![allow(clippy::enum_variant_names)]
    #![allow(dead_code)]
    #![allow(clippy::type_complexity)]
    #![allow(unused_imports)]
    use ethers::contract::{
        builders::{ContractCall, Event},
        Contract, Lazy,
    };
    use ethers::core::{
        abi::{Abi, Detokenize, InvalidOutputType, Token, Tokenizable},
        types::*,
    };
    use ethers::providers::Middleware;
    #[doc = "HoprWrapper was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    # [rustfmt :: skip] const __ABI : & str = "[{\"inputs\":[{\"internalType\":\"contract IERC20\",\"name\":\"_xHOPR\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"contract HoprToken\",\"name\":\"_wxHOPR\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"constructor\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"previousOwner\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"newOwner\",\"type\":\"address\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"OwnershipTransferred\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"Unwrapped\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"Wrapped\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"TOKENS_RECIPIENT_INTERFACE_HASH\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"interfaceHash\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"canImplementInterfaceForAddress\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"from\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"data\",\"type\":\"bytes\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"onTokenTransfer\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"success\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"owner\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"recoverTokens\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"renounceOwnership\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"from\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"userData\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"operatorData\",\"type\":\"bytes\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"tokensReceived\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"newOwner\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"transferOwnership\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"wxHOPR\",\"outputs\":[{\"internalType\":\"contract HoprToken\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"xHOPR\",\"outputs\":[{\"internalType\":\"contract IERC20\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"xHoprAmount\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]}]" ;
    #[doc = r" The parsed JSON-ABI of the contract."]
    pub static HOPRWRAPPER_ABI: ethers::contract::Lazy<ethers::core::abi::Abi> =
        ethers::contract::Lazy::new(|| {
            ethers::core::utils::__serde_json::from_str(__ABI).expect("invalid abi")
        });
    #[doc = r" Bytecode of the #name contract"]
    pub static HOPRWRAPPER_BYTECODE: ethers::contract::Lazy<ethers::core::types::Bytes> =
        ethers::contract::Lazy::new(|| {
            "0x6080604052600060035534801561001557600080fd5b50604051610fd6380380610fd68339818101604052604081101561003857600080fd5b508051602090910151600061004b61016d565b600080546001600160a01b0319166001600160a01b0383169081178255604051929350917f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e0908290a3506002805460ff19166001179055600480546001600160a01b038085166001600160a01b03199283161783556005805491851691909216179055604080516329965a1d60e01b8152309281018390527fb281fc8c12954d22544db45de3159a39272895b169a852b314f9cc762e44c53b6024820152604481019290925251731820a4b7618bde71dce8cdc73aab6c95905fad24916329965a1d91606480830192600092919082900301818387803b15801561014e57600080fd5b505af1158015610162573d6000803e3d6000fd5b505050505050610171565b3390565b610e56806101806000396000f3fe608060405234801561001057600080fd5b50600436106100a85760003560e01c80638da5cb5b116100715780638da5cb5b1461020b578063a4c0ed3614610213578063b77f39fe146102ac578063d9a465aa146102b4578063de279afe146102bc578063f2fde38b146102c4576100a8565b806223de29146100ad5780631a5518b114610199578063249cb3fa146101bd578063715018a6146101fb57806372581cc014610203575b600080fd5b610197600480360360c08110156100c357600080fd5b6001600160a01b03823581169260208101358216926040820135909216916060820135919081019060a08101608082013564010000000081111561010657600080fd5b82018360208201111561011857600080fd5b8035906020019184600183028401116401000000008311171561013a57600080fd5b91939092909160208101903564010000000081111561015857600080fd5b82018360208201111561016a57600080fd5b8035906020019184600183028401116401000000008311171561018c57600080fd5b5090925090506102ea565b005b6101a16104d9565b604080516001600160a01b039092168252519081900360200190f35b6101e9600480360360408110156101d357600080fd5b50803590602001356001600160a01b03166104e8565b60408051918252519081900360200190f35b61019761055f565b6101e9610613565b6101a1610637565b6102986004803603606081101561022957600080fd5b6001600160a01b038235169160208101359181019060608101604082013564010000000081111561025957600080fd5b82018360208201111561026b57600080fd5b8035906020019184600183028401116401000000008311171561028d57600080fd5b509092509050610646565b604080519115158252519081900360200190f35b6101976107ef565b6101a1610904565b6101e9610913565b610197600480360360208110156102da57600080fd5b50356001600160a01b0316610919565b60025460ff16610341576040805162461bcd60e51b815260206004820152601f60248201527f5265656e7472616e637947756172643a207265656e7472616e742063616c6c00604482015290519081900360640190fd5b6002805460ff191690556005546001600160a01b031633146103a2576040805162461bcd60e51b815260206004820152601560248201527429b2b73232b91036bab9ba103132903bbc2427a82960591b604482015290519081900360640190fd5b6001600160a01b03861630146103e95760405162461bcd60e51b8152600401808060200182810382526025815260200180610dd26025913960400191505060405180910390fd5b6003546103f69086610a23565b6003556005546040805163fe9d930360e01b8152600481018890526024810182905260006044820181905291516001600160a01b039093169263fe9d93039260848084019391929182900301818387803b15801561045357600080fd5b505af1158015610467573d6000803e3d6000fd5b505060045461048392506001600160a01b031690508887610a65565b6040805186815290516001600160a01b038916917f95ae649bfaaef9def56a52f4fb2d9e8fa5496bb7082930e442c74cc76b03dcb3919081900360200190a250506002805460ff19166001179055505050505050565b6004546001600160a01b031681565b60008281526001602090815260408083206001600160a01b038516845290915281205460ff16610519576000610558565b604051602001808073455243313832305f4143434550545f4d4147494360601b8152506014019050604051602081830303815290604052805190602001205b9392505050565b610567610abc565b6000546001600160a01b039081169116146105c9576040805162461bcd60e51b815260206004820181905260248201527f4f776e61626c653a2063616c6c6572206973206e6f7420746865206f776e6572604482015290519081900360640190fd5b600080546040516001600160a01b03909116907f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e0908390a3600080546001600160a01b0319169055565b7fb281fc8c12954d22544db45de3159a39272895b169a852b314f9cc762e44c53b81565b6000546001600160a01b031690565b60025460009060ff166106a0576040805162461bcd60e51b815260206004820152601f60248201527f5265656e7472616e637947756172643a207265656e7472616e742063616c6c00604482015290519081900360640190fd5b6002805460ff191690556004546001600160a01b03163314610700576040805162461bcd60e51b815260206004820152601460248201527329b2b73232b91036bab9ba103132903c2427a82960611b604482015290519081900360640190fd5b60035461070d9085610ac0565b60035560055460408051630dcdc7dd60e41b81526001600160a01b038881166004830152602482018890526080604483015260006084830181905260c0606484015260c48301819052925193169263dcdc7dd0926101048084019391929182900301818387803b15801561078057600080fd5b505af1158015610794573d6000803e3d6000fd5b50506040805187815290516001600160a01b03891693507f4700c1726b4198077cd40320a32c45265a1910521eb0ef713dd1d8412413d7fc92509081900360200190a25060016002805460ff19166001179055949350505050565b6107f7610abc565b6000546001600160a01b03908116911614610859576040805162461bcd60e51b815260206004820181905260248201527f4f776e61626c653a2063616c6c6572206973206e6f7420746865206f776e6572604482015290519081900360640190fd5b60035460048054604080516370a0823160e01b81523093810193909352516000936108e29390926001600160a01b0316916370a0823191602480820192602092909190829003018186803b1580156108b057600080fd5b505afa1580156108c4573d6000803e3d6000fd5b505050506040513d60208110156108da57600080fd5b505190610a23565b9050801561090157600454610901906001600160a01b03163383610a65565b50565b6005546001600160a01b031681565b60035481565b610921610abc565b6000546001600160a01b03908116911614610983576040805162461bcd60e51b815260206004820181905260248201527f4f776e61626c653a2063616c6c6572206973206e6f7420746865206f776e6572604482015290519081900360640190fd5b6001600160a01b0381166109c85760405162461bcd60e51b8152600401808060200182810382526026815260200180610dac6026913960400191505060405180910390fd5b600080546040516001600160a01b03808516939216917f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e091a3600080546001600160a01b0319166001600160a01b0392909216919091179055565b600061055883836040518060400160405280601e81526020017f536166654d6174683a207375627472616374696f6e206f766572666c6f770000815250610b1a565b604080516001600160a01b038416602482015260448082018490528251808303909101815260649091019091526020810180516001600160e01b031663a9059cbb60e01b179052610ab7908490610bb1565b505050565b3390565b600082820183811015610558576040805162461bcd60e51b815260206004820152601b60248201527f536166654d6174683a206164646974696f6e206f766572666c6f770000000000604482015290519081900360640190fd5b60008184841115610ba95760405162461bcd60e51b81526004018080602001828103825283818151815260200191508051906020019080838360005b83811015610b6e578181015183820152602001610b56565b50505050905090810190601f168015610b9b5780820380516001836020036101000a031916815260200191505b509250505060405180910390fd5b505050900390565b610bc3826001600160a01b0316610d6f565b610c14576040805162461bcd60e51b815260206004820152601f60248201527f5361666545524332303a2063616c6c20746f206e6f6e2d636f6e747261637400604482015290519081900360640190fd5b60006060836001600160a01b0316836040518082805190602001908083835b60208310610c525780518252601f199092019160209182019101610c33565b6001836020036101000a0380198251168184511680821785525050505050509050019150506000604051808303816000865af19150503d8060008114610cb4576040519150601f19603f3d011682016040523d82523d6000602084013e610cb9565b606091505b509150915081610d10576040805162461bcd60e51b815260206004820181905260248201527f5361666545524332303a206c6f772d6c6576656c2063616c6c206661696c6564604482015290519081900360640190fd5b805115610d6957808060200190516020811015610d2c57600080fd5b5051610d695760405162461bcd60e51b815260040180806020018281038252602a815260200180610df7602a913960400191505060405180910390fd5b50505050565b6000813f7fc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470818114801590610da357508115155b94935050505056fe4f776e61626c653a206e6577206f776e657220697320746865207a65726f20616464726573734d7573742062652073656e64696e6720746f6b656e7320746f20486f7072577261707065725361666545524332303a204552433230206f7065726174696f6e20646964206e6f742073756363656564a2646970667358221220e115261aa3c816805cf71378410043325050da679088bac73e8af20aa4f69b1464736f6c634300060c0033" . parse () . expect ("invalid bytecode")
        });
    pub struct HoprWrapper<M>(ethers::contract::Contract<M>);
    impl<M> Clone for HoprWrapper<M> {
        fn clone(&self) -> Self {
            HoprWrapper(self.0.clone())
        }
    }
    impl<M> std::ops::Deref for HoprWrapper<M> {
        type Target = ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> std::fmt::Debug for HoprWrapper<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(HoprWrapper))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ethers::providers::Middleware> HoprWrapper<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            ethers::contract::Contract::new(address.into(), HOPRWRAPPER_ABI.clone(), client).into()
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
                HOPRWRAPPER_ABI.clone(),
                HOPRWRAPPER_BYTECODE.clone().into(),
                client,
            );
            let deployer = factory.deploy(constructor_args)?;
            let deployer = ethers::contract::ContractDeployer::new(deployer);
            Ok(deployer)
        }
        #[doc = "Calls the contract's `TOKENS_RECIPIENT_INTERFACE_HASH` (0x72581cc0) function"]
        pub fn tokens_recipient_interface_hash(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([114, 88, 28, 192], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `canImplementInterfaceForAddress` (0x249cb3fa) function"]
        pub fn can_implement_interface_for_address(
            &self,
            interface_hash: [u8; 32],
            account: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([36, 156, 179, 250], (interface_hash, account))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `onTokenTransfer` (0xa4c0ed36) function"]
        pub fn on_token_transfer(
            &self,
            from: ethers::core::types::Address,
            amount: ethers::core::types::U256,
            data: ethers::core::types::Bytes,
        ) -> ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([164, 192, 237, 54], (from, amount, data))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `owner` (0x8da5cb5b) function"]
        pub fn owner(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Address> {
            self.0
                .method_hash([141, 165, 203, 91], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `recoverTokens` (0xb77f39fe) function"]
        pub fn recover_tokens(&self) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([183, 127, 57, 254], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `renounceOwnership` (0x715018a6) function"]
        pub fn renounce_ownership(&self) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([113, 80, 24, 166], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `tokensReceived` (0x0023de29) function"]
        pub fn tokens_received(
            &self,
            operator: ethers::core::types::Address,
            from: ethers::core::types::Address,
            to: ethers::core::types::Address,
            amount: ethers::core::types::U256,
            user_data: ethers::core::types::Bytes,
            operator_data: ethers::core::types::Bytes,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [0, 35, 222, 41],
                    (operator, from, to, amount, user_data, operator_data),
                )
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `transferOwnership` (0xf2fde38b) function"]
        pub fn transfer_ownership(
            &self,
            new_owner: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([242, 253, 227, 139], new_owner)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `wxHOPR` (0xd9a465aa) function"]
        pub fn wx_hopr(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Address> {
            self.0
                .method_hash([217, 164, 101, 170], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `xHOPR` (0x1a5518b1) function"]
        pub fn x_hopr(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Address> {
            self.0
                .method_hash([26, 85, 24, 177], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `xHoprAmount` (0xde279afe) function"]
        pub fn x_hopr_amount(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([222, 39, 154, 254], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Gets the contract's `OwnershipTransferred` event"]
        pub fn ownership_transferred_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, OwnershipTransferredFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `Unwrapped` event"]
        pub fn unwrapped_filter(&self) -> ethers::contract::builders::Event<M, UnwrappedFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `Wrapped` event"]
        pub fn wrapped_filter(&self) -> ethers::contract::builders::Event<M, WrappedFilter> {
            self.0.event()
        }
        #[doc = r" Returns an [`Event`](#ethers_contract::builders::Event) builder for all events of this contract"]
        pub fn events(&self) -> ethers::contract::builders::Event<M, HoprWrapperEvents> {
            self.0.event_with_filter(Default::default())
        }
    }
    impl<M: ethers::providers::Middleware> From<ethers::contract::Contract<M>> for HoprWrapper<M> {
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
    #[ethevent(
        name = "OwnershipTransferred",
        abi = "OwnershipTransferred(address,address)"
    )]
    pub struct OwnershipTransferredFilter {
        #[ethevent(indexed)]
        pub previous_owner: ethers::core::types::Address,
        #[ethevent(indexed)]
        pub new_owner: ethers::core::types::Address,
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
    #[ethevent(name = "Unwrapped", abi = "Unwrapped(address,uint256)")]
    pub struct UnwrappedFilter {
        #[ethevent(indexed)]
        pub account: ethers::core::types::Address,
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
    #[ethevent(name = "Wrapped", abi = "Wrapped(address,uint256)")]
    pub struct WrappedFilter {
        #[ethevent(indexed)]
        pub account: ethers::core::types::Address,
        pub amount: ethers::core::types::U256,
    }
    #[derive(Debug, Clone, PartialEq, Eq, ethers :: contract :: EthAbiType)]
    pub enum HoprWrapperEvents {
        OwnershipTransferredFilter(OwnershipTransferredFilter),
        UnwrappedFilter(UnwrappedFilter),
        WrappedFilter(WrappedFilter),
    }
    impl ethers::contract::EthLogDecode for HoprWrapperEvents {
        fn decode_log(
            log: &ethers::core::abi::RawLog,
        ) -> ::std::result::Result<Self, ethers::core::abi::Error>
        where
            Self: Sized,
        {
            if let Ok(decoded) = OwnershipTransferredFilter::decode_log(log) {
                return Ok(HoprWrapperEvents::OwnershipTransferredFilter(decoded));
            }
            if let Ok(decoded) = UnwrappedFilter::decode_log(log) {
                return Ok(HoprWrapperEvents::UnwrappedFilter(decoded));
            }
            if let Ok(decoded) = WrappedFilter::decode_log(log) {
                return Ok(HoprWrapperEvents::WrappedFilter(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData)
        }
    }
    impl ::std::fmt::Display for HoprWrapperEvents {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                HoprWrapperEvents::OwnershipTransferredFilter(element) => element.fmt(f),
                HoprWrapperEvents::UnwrappedFilter(element) => element.fmt(f),
                HoprWrapperEvents::WrappedFilter(element) => element.fmt(f),
            }
        }
    }
    #[doc = "Container type for all input parameters for the `TOKENS_RECIPIENT_INTERFACE_HASH` function with signature `TOKENS_RECIPIENT_INTERFACE_HASH()` and selector `[114, 88, 28, 192]`"]
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
        name = "TOKENS_RECIPIENT_INTERFACE_HASH",
        abi = "TOKENS_RECIPIENT_INTERFACE_HASH()"
    )]
    pub struct TokensRecipientInterfaceHashCall;
    #[doc = "Container type for all input parameters for the `canImplementInterfaceForAddress` function with signature `canImplementInterfaceForAddress(bytes32,address)` and selector `[36, 156, 179, 250]`"]
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
        name = "canImplementInterfaceForAddress",
        abi = "canImplementInterfaceForAddress(bytes32,address)"
    )]
    pub struct CanImplementInterfaceForAddressCall {
        pub interface_hash: [u8; 32],
        pub account: ethers::core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `onTokenTransfer` function with signature `onTokenTransfer(address,uint256,bytes)` and selector `[164, 192, 237, 54]`"]
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
        name = "onTokenTransfer",
        abi = "onTokenTransfer(address,uint256,bytes)"
    )]
    pub struct OnTokenTransferCall {
        pub from: ethers::core::types::Address,
        pub amount: ethers::core::types::U256,
        pub data: ethers::core::types::Bytes,
    }
    #[doc = "Container type for all input parameters for the `owner` function with signature `owner()` and selector `[141, 165, 203, 91]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "owner", abi = "owner()")]
    pub struct OwnerCall;
    #[doc = "Container type for all input parameters for the `recoverTokens` function with signature `recoverTokens()` and selector `[183, 127, 57, 254]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "recoverTokens", abi = "recoverTokens()")]
    pub struct RecoverTokensCall;
    #[doc = "Container type for all input parameters for the `renounceOwnership` function with signature `renounceOwnership()` and selector `[113, 80, 24, 166]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "renounceOwnership", abi = "renounceOwnership()")]
    pub struct RenounceOwnershipCall;
    #[doc = "Container type for all input parameters for the `tokensReceived` function with signature `tokensReceived(address,address,address,uint256,bytes,bytes)` and selector `[0, 35, 222, 41]`"]
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
        name = "tokensReceived",
        abi = "tokensReceived(address,address,address,uint256,bytes,bytes)"
    )]
    pub struct TokensReceivedCall {
        pub operator: ethers::core::types::Address,
        pub from: ethers::core::types::Address,
        pub to: ethers::core::types::Address,
        pub amount: ethers::core::types::U256,
        pub user_data: ethers::core::types::Bytes,
        pub operator_data: ethers::core::types::Bytes,
    }
    #[doc = "Container type for all input parameters for the `transferOwnership` function with signature `transferOwnership(address)` and selector `[242, 253, 227, 139]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "transferOwnership", abi = "transferOwnership(address)")]
    pub struct TransferOwnershipCall {
        pub new_owner: ethers::core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `wxHOPR` function with signature `wxHOPR()` and selector `[217, 164, 101, 170]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "wxHOPR", abi = "wxHOPR()")]
    pub struct WxHOPRCall;
    #[doc = "Container type for all input parameters for the `xHOPR` function with signature `xHOPR()` and selector `[26, 85, 24, 177]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "xHOPR", abi = "xHOPR()")]
    pub struct XhoprCall;
    #[doc = "Container type for all input parameters for the `xHoprAmount` function with signature `xHoprAmount()` and selector `[222, 39, 154, 254]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "xHoprAmount", abi = "xHoprAmount()")]
    pub struct XhoprAmountCall;
    #[derive(Debug, Clone, PartialEq, Eq, ethers :: contract :: EthAbiType)]
    pub enum HoprWrapperCalls {
        TokensRecipientInterfaceHash(TokensRecipientInterfaceHashCall),
        CanImplementInterfaceForAddress(CanImplementInterfaceForAddressCall),
        OnTokenTransfer(OnTokenTransferCall),
        Owner(OwnerCall),
        RecoverTokens(RecoverTokensCall),
        RenounceOwnership(RenounceOwnershipCall),
        TokensReceived(TokensReceivedCall),
        TransferOwnership(TransferOwnershipCall),
        WxHOPR(WxHOPRCall),
        Xhopr(XhoprCall),
        XhoprAmount(XhoprAmountCall),
    }
    impl ethers::core::abi::AbiDecode for HoprWrapperCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::std::result::Result<Self, ethers::core::abi::AbiError> {
            if let Ok(decoded) =
                <TokensRecipientInterfaceHashCall as ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                )
            {
                return Ok(HoprWrapperCalls::TokensRecipientInterfaceHash(decoded));
            }
            if let Ok(decoded) =
                <CanImplementInterfaceForAddressCall as ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                )
            {
                return Ok(HoprWrapperCalls::CanImplementInterfaceForAddress(decoded));
            }
            if let Ok(decoded) =
                <OnTokenTransferCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprWrapperCalls::OnTokenTransfer(decoded));
            }
            if let Ok(decoded) = <OwnerCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprWrapperCalls::Owner(decoded));
            }
            if let Ok(decoded) =
                <RecoverTokensCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprWrapperCalls::RecoverTokens(decoded));
            }
            if let Ok(decoded) =
                <RenounceOwnershipCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprWrapperCalls::RenounceOwnership(decoded));
            }
            if let Ok(decoded) =
                <TokensReceivedCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprWrapperCalls::TokensReceived(decoded));
            }
            if let Ok(decoded) =
                <TransferOwnershipCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprWrapperCalls::TransferOwnership(decoded));
            }
            if let Ok(decoded) = <WxHOPRCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprWrapperCalls::WxHOPR(decoded));
            }
            if let Ok(decoded) = <XhoprCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprWrapperCalls::Xhopr(decoded));
            }
            if let Ok(decoded) =
                <XhoprAmountCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprWrapperCalls::XhoprAmount(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ethers::core::abi::AbiEncode for HoprWrapperCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                HoprWrapperCalls::TokensRecipientInterfaceHash(element) => element.encode(),
                HoprWrapperCalls::CanImplementInterfaceForAddress(element) => element.encode(),
                HoprWrapperCalls::OnTokenTransfer(element) => element.encode(),
                HoprWrapperCalls::Owner(element) => element.encode(),
                HoprWrapperCalls::RecoverTokens(element) => element.encode(),
                HoprWrapperCalls::RenounceOwnership(element) => element.encode(),
                HoprWrapperCalls::TokensReceived(element) => element.encode(),
                HoprWrapperCalls::TransferOwnership(element) => element.encode(),
                HoprWrapperCalls::WxHOPR(element) => element.encode(),
                HoprWrapperCalls::Xhopr(element) => element.encode(),
                HoprWrapperCalls::XhoprAmount(element) => element.encode(),
            }
        }
    }
    impl ::std::fmt::Display for HoprWrapperCalls {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                HoprWrapperCalls::TokensRecipientInterfaceHash(element) => element.fmt(f),
                HoprWrapperCalls::CanImplementInterfaceForAddress(element) => element.fmt(f),
                HoprWrapperCalls::OnTokenTransfer(element) => element.fmt(f),
                HoprWrapperCalls::Owner(element) => element.fmt(f),
                HoprWrapperCalls::RecoverTokens(element) => element.fmt(f),
                HoprWrapperCalls::RenounceOwnership(element) => element.fmt(f),
                HoprWrapperCalls::TokensReceived(element) => element.fmt(f),
                HoprWrapperCalls::TransferOwnership(element) => element.fmt(f),
                HoprWrapperCalls::WxHOPR(element) => element.fmt(f),
                HoprWrapperCalls::Xhopr(element) => element.fmt(f),
                HoprWrapperCalls::XhoprAmount(element) => element.fmt(f),
            }
        }
    }
    impl ::std::convert::From<TokensRecipientInterfaceHashCall> for HoprWrapperCalls {
        fn from(var: TokensRecipientInterfaceHashCall) -> Self {
            HoprWrapperCalls::TokensRecipientInterfaceHash(var)
        }
    }
    impl ::std::convert::From<CanImplementInterfaceForAddressCall> for HoprWrapperCalls {
        fn from(var: CanImplementInterfaceForAddressCall) -> Self {
            HoprWrapperCalls::CanImplementInterfaceForAddress(var)
        }
    }
    impl ::std::convert::From<OnTokenTransferCall> for HoprWrapperCalls {
        fn from(var: OnTokenTransferCall) -> Self {
            HoprWrapperCalls::OnTokenTransfer(var)
        }
    }
    impl ::std::convert::From<OwnerCall> for HoprWrapperCalls {
        fn from(var: OwnerCall) -> Self {
            HoprWrapperCalls::Owner(var)
        }
    }
    impl ::std::convert::From<RecoverTokensCall> for HoprWrapperCalls {
        fn from(var: RecoverTokensCall) -> Self {
            HoprWrapperCalls::RecoverTokens(var)
        }
    }
    impl ::std::convert::From<RenounceOwnershipCall> for HoprWrapperCalls {
        fn from(var: RenounceOwnershipCall) -> Self {
            HoprWrapperCalls::RenounceOwnership(var)
        }
    }
    impl ::std::convert::From<TokensReceivedCall> for HoprWrapperCalls {
        fn from(var: TokensReceivedCall) -> Self {
            HoprWrapperCalls::TokensReceived(var)
        }
    }
    impl ::std::convert::From<TransferOwnershipCall> for HoprWrapperCalls {
        fn from(var: TransferOwnershipCall) -> Self {
            HoprWrapperCalls::TransferOwnership(var)
        }
    }
    impl ::std::convert::From<WxHOPRCall> for HoprWrapperCalls {
        fn from(var: WxHOPRCall) -> Self {
            HoprWrapperCalls::WxHOPR(var)
        }
    }
    impl ::std::convert::From<XhoprCall> for HoprWrapperCalls {
        fn from(var: XhoprCall) -> Self {
            HoprWrapperCalls::Xhopr(var)
        }
    }
    impl ::std::convert::From<XhoprAmountCall> for HoprWrapperCalls {
        fn from(var: XhoprAmountCall) -> Self {
            HoprWrapperCalls::XhoprAmount(var)
        }
    }
    #[doc = "Container type for all return fields from the `TOKENS_RECIPIENT_INTERFACE_HASH` function with signature `TOKENS_RECIPIENT_INTERFACE_HASH()` and selector `[114, 88, 28, 192]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct TokensRecipientInterfaceHashReturn(pub [u8; 32]);
    #[doc = "Container type for all return fields from the `canImplementInterfaceForAddress` function with signature `canImplementInterfaceForAddress(bytes32,address)` and selector `[36, 156, 179, 250]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct CanImplementInterfaceForAddressReturn(pub [u8; 32]);
    #[doc = "Container type for all return fields from the `onTokenTransfer` function with signature `onTokenTransfer(address,uint256,bytes)` and selector `[164, 192, 237, 54]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct OnTokenTransferReturn {
        pub success: bool,
    }
    #[doc = "Container type for all return fields from the `owner` function with signature `owner()` and selector `[141, 165, 203, 91]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct OwnerReturn(pub ethers::core::types::Address);
    #[doc = "Container type for all return fields from the `wxHOPR` function with signature `wxHOPR()` and selector `[217, 164, 101, 170]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct WxHOPRReturn(pub ethers::core::types::Address);
    #[doc = "Container type for all return fields from the `xHOPR` function with signature `xHOPR()` and selector `[26, 85, 24, 177]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct XhoprReturn(pub ethers::core::types::Address);
    #[doc = "Container type for all return fields from the `xHoprAmount` function with signature `xHoprAmount()` and selector `[222, 39, 154, 254]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct XhoprAmountReturn(pub ethers::core::types::U256);
}
