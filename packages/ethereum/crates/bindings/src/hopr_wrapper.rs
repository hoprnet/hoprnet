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
            "0x6080604052600060035534801561001557600080fd5b50604051610f03380380610f0383398101604081905261003491610170565b61003d33610108565b6001600255600480546001600160a01b038481166001600160a01b031992831617835560058054918516919092161790556040516329965a1d60e01b8152309181018290527fb281fc8c12954d22544db45de3159a39272895b169a852b314f9cc762e44c53b60248201526044810191909152731820a4b7618bde71dce8cdc73aab6c95905fad24906329965a1d90606401600060405180830381600087803b1580156100e957600080fd5b505af11580156100fd573d6000803e3d6000fd5b5050505050506101aa565b600080546001600160a01b038381166001600160a01b0319831681178455604051919092169283917f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e09190a35050565b6001600160a01b038116811461016d57600080fd5b50565b6000806040838503121561018357600080fd5b825161018e81610158565b602084015190925061019f81610158565b809150509250929050565b610d4a806101b96000396000f3fe608060405234801561001057600080fd5b50600436106100a85760003560e01c80638da5cb5b116100715780638da5cb5b14610142578063a4c0ed3614610153578063b77f39fe14610176578063d9a465aa1461017e578063de279afe14610191578063f2fde38b1461019a57600080fd5b806223de29146100ad5780631a5518b1146100c2578063249cb3fa146100f2578063715018a61461011357806372581cc01461011b575b600080fd5b6100c06100bb366004610a94565b6101ad565b005b6004546100d5906001600160a01b031681565b6040516001600160a01b0390911681526020015b60405180910390f35b610105610100366004610b3f565b6103a7565b6040519081526020016100e9565b6100c0610401565b6101057fb281fc8c12954d22544db45de3159a39272895b169a852b314f9cc762e44c53b81565b6000546001600160a01b03166100d5565b610166610161366004610b6b565b610437565b60405190151581526020016100e9565b6100c06105c3565b6005546100d5906001600160a01b031681565b61010560035481565b6100c06101a8366004610bc5565b61068e565b60028054036102035760405162461bcd60e51b815260206004820152601f60248201527f5265656e7472616e637947756172643a207265656e7472616e742063616c6c0060448201526064015b60405180910390fd5b600280556005546001600160a01b031633146102595760405162461bcd60e51b815260206004820152601560248201527429b2b73232b91036bab9ba103132903bbc2427a82960591b60448201526064016101fa565b6001600160a01b03861630146102bf5760405162461bcd60e51b815260206004820152602560248201527f4d7573742062652073656e64696e6720746f6b656e7320746f20486f7072577260448201526430b83832b960d91b60648201526084016101fa565b6003546102cc9086610726565b6003556005546040805163fe9d930360e01b8152600481018890526024810191909152600060448201526001600160a01b039091169063fe9d930390606401600060405180830381600087803b15801561032557600080fd5b505af1158015610339573d6000803e3d6000fd5b505060045461035592506001600160a01b031690508887610732565b866001600160a01b03167f95ae649bfaaef9def56a52f4fb2d9e8fa5496bb7082930e442c74cc76b03dcb38660405161039091815260200190565b60405180910390a250506001600255505050505050565b60008281526001602090815260408083206001600160a01b038516845290915281205460ff166103d85760006103fa565b7fa2ef4600d742022d532d4747cb3547474667d6f13804902513b2ec01c848f4b45b9392505050565b6000546001600160a01b0316331461042b5760405162461bcd60e51b81526004016101fa90610be0565b6104356000610789565b565b6000600280540361048a5760405162461bcd60e51b815260206004820152601f60248201527f5265656e7472616e637947756172643a207265656e7472616e742063616c6c0060448201526064016101fa565b600280556004546001600160a01b031633146104df5760405162461bcd60e51b815260206004820152601460248201527329b2b73232b91036bab9ba103132903c2427a82960611b60448201526064016101fa565b6003546104ec90856107d9565b600355600554604051630dcdc7dd60e41b81526001600160a01b038781166004830152602482018790526080604483015260006084830181905260a0606484015260a48301529091169063dcdc7dd09060c401600060405180830381600087803b15801561055957600080fd5b505af115801561056d573d6000803e3d6000fd5b50505050846001600160a01b03167f4700c1726b4198077cd40320a32c45265a1910521eb0ef713dd1d8412413d7fc856040516105ac91815260200190565b60405180910390a250600180600255949350505050565b6000546001600160a01b031633146105ed5760405162461bcd60e51b81526004016101fa90610be0565b600354600480546040516370a0823160e01b8152309281019290925260009261066c9290916001600160a01b0316906370a0823190602401602060405180830381865afa158015610642573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906106669190610c15565b90610726565b9050801561068b5760045461068b906001600160a01b03163383610732565b50565b6000546001600160a01b031633146106b85760405162461bcd60e51b81526004016101fa90610be0565b6001600160a01b03811661071d5760405162461bcd60e51b815260206004820152602660248201527f4f776e61626c653a206e6577206f776e657220697320746865207a65726f206160448201526564647265737360d01b60648201526084016101fa565b61068b81610789565b60006103fa8284610c44565b604080516001600160a01b038416602482015260448082018490528251808303909101815260649091019091526020810180516001600160e01b031663a9059cbb60e01b1790526107849084906107e5565b505050565b600080546001600160a01b038381166001600160a01b0319831681178455604051919092169283917f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e09190a35050565b60006103fa8284610c5b565b600061083a826040518060400160405280602081526020017f5361666545524332303a206c6f772d6c6576656c2063616c6c206661696c6564815250856001600160a01b03166108b79092919063ffffffff16565b80519091501561078457808060200190518101906108589190610c73565b6107845760405162461bcd60e51b815260206004820152602a60248201527f5361666545524332303a204552433230206f7065726174696f6e20646964206e6044820152691bdd081cdd58d8d9595960b21b60648201526084016101fa565b60606108c684846000856108ce565b949350505050565b60608247101561092f5760405162461bcd60e51b815260206004820152602660248201527f416464726573733a20696e73756666696369656e742062616c616e636520666f6044820152651c8818d85b1b60d21b60648201526084016101fa565b843b61097d5760405162461bcd60e51b815260206004820152601d60248201527f416464726573733a2063616c6c20746f206e6f6e2d636f6e747261637400000060448201526064016101fa565b600080866001600160a01b031685876040516109999190610cc5565b60006040518083038185875af1925050503d80600081146109d6576040519150601f19603f3d011682016040523d82523d6000602084013e6109db565b606091505b50915091506109eb8282866109f6565b979650505050505050565b60608315610a055750816103fa565b825115610a155782518084602001fd5b8160405162461bcd60e51b81526004016101fa9190610ce1565b80356001600160a01b0381168114610a4657600080fd5b919050565b60008083601f840112610a5d57600080fd5b50813567ffffffffffffffff811115610a7557600080fd5b602083019150836020828501011115610a8d57600080fd5b9250929050565b60008060008060008060008060c0898b031215610ab057600080fd5b610ab989610a2f565b9750610ac760208a01610a2f565b9650610ad560408a01610a2f565b955060608901359450608089013567ffffffffffffffff80821115610af957600080fd5b610b058c838d01610a4b565b909650945060a08b0135915080821115610b1e57600080fd5b50610b2b8b828c01610a4b565b999c989b5096995094979396929594505050565b60008060408385031215610b5257600080fd5b82359150610b6260208401610a2f565b90509250929050565b60008060008060608587031215610b8157600080fd5b610b8a85610a2f565b935060208501359250604085013567ffffffffffffffff811115610bad57600080fd5b610bb987828801610a4b565b95989497509550505050565b600060208284031215610bd757600080fd5b6103fa82610a2f565b6020808252818101527f4f776e61626c653a2063616c6c6572206973206e6f7420746865206f776e6572604082015260600190565b600060208284031215610c2757600080fd5b5051919050565b634e487b7160e01b600052601160045260246000fd5b600082821015610c5657610c56610c2e565b500390565b60008219821115610c6e57610c6e610c2e565b500190565b600060208284031215610c8557600080fd5b815180151581146103fa57600080fd5b60005b83811015610cb0578181015183820152602001610c98565b83811115610cbf576000848401525b50505050565b60008251610cd7818460208701610c95565b9190910192915050565b6020815260008251806020840152610d00816040850160208701610c95565b601f01601f1916919091016040019291505056fea26469706673582212200248aff230481269758039c85e0daae40f6cda6ca6e19eb2e3702f64973a642364736f6c634300080d0033" . parse () . expect ("invalid bytecode")
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
