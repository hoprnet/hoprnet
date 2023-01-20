pub use hopr_dummy_proxy_for_network_registry::*;
#[allow(clippy::too_many_arguments, non_camel_case_types)]
pub mod hopr_dummy_proxy_for_network_registry {
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
    #[doc = "HoprDummyProxyForNetworkRegistry was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    # [rustfmt :: skip] const __ABI : & str = "[{\"inputs\":[{\"internalType\":\"address\",\"name\":\"newOwner\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"constructor\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"AccountDeregistered\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"AccountRegistered\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"bool\",\"name\":\"isAllowed\",\"type\":\"bool\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"AllowAllAccountsEligible\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"previousOwner\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"newOwner\",\"type\":\"address\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"OwnershipTransferred\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"MAX_REGISTRATION_PER_ACCOUNT\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"isAllAllowed\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"maxAllowedRegistrations\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"owner\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"ownerAddAccount\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address[]\",\"name\":\"accounts\",\"type\":\"address[]\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"ownerBatchAddAccounts\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address[]\",\"name\":\"accounts\",\"type\":\"address[]\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"ownerBatchRemoveAccounts\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"ownerRemoveAccount\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"renounceOwnership\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"newOwner\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"transferOwnership\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"bool\",\"name\":\"_updatedAllow\",\"type\":\"bool\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"updateAllowAll\",\"outputs\":[]}]" ;
    #[doc = r" The parsed JSON-ABI of the contract."]
    pub static HOPRDUMMYPROXYFORNETWORKREGISTRY_ABI: ethers::contract::Lazy<
        ethers::core::abi::Abi,
    > = ethers::contract::Lazy::new(|| {
        ethers::core::utils::__serde_json::from_str(__ABI).expect("invalid abi")
    });
    #[doc = r" Bytecode of the #name contract"]
    pub static HOPRDUMMYPROXYFORNETWORKREGISTRY_BYTECODE: ethers::contract::Lazy<
        ethers::core::types::Bytes,
    > = ethers::contract::Lazy::new(|| {
        "0x608060405234801561001057600080fd5b5060405161082f38038061082f83398101604081905261002f916100d5565b61003833610085565b61004181610085565b6002805460ff19169055604051600081527fafab23a4bc8c49250ba37eeb0625b0a9b271f55d1501838d24f54508c3b173429060200160405180910390a150610105565b600080546001600160a01b038381166001600160a01b0319831681178455604051919092169283917f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e09190a35050565b6000602082840312156100e757600080fd5b81516001600160a01b03811681146100fe57600080fd5b9392505050565b61071b806101146000396000f3fe608060405234801561001057600080fd5b50600436106100a95760003560e01c80638da5cb5b116100715780638da5cb5b14610104578063a71b1b7414610124578063b3544e8214610137578063f2fde38b14610158578063f46c84b81461016b578063f67f5e6f1461017457600080fd5b8063089ffa47146100ae5780634051f257146100c3578063486354e2146100d65780635c5471e7146100e9578063715018a6146100fc575b600080fd5b6100c16100bc3660046105ac565b610191565b005b6100c16100d13660046105d5565b61021c565b6100c16100e43660046105fe565b61024f565b6100c16100f73660046105fe565b6102ca565b6100c1610340565b6000546040516001600160a01b0390911681526020015b60405180910390f35b6100c16101323660046105d5565b610376565b61014a6101453660046105d5565b6103a9565b60405190815260200161011b565b6100c16101663660046105d5565b6103eb565b61014a60001981565b6002546101819060ff1681565b604051901515815260200161011b565b6000546001600160a01b031633146101c45760405162461bcd60e51b81526004016101bb90610673565b60405180910390fd5b60025460ff16151581151514610219576002805460ff19168215159081179091556040519081527fafab23a4bc8c49250ba37eeb0625b0a9b271f55d1501838d24f54508c3b173429060200160405180910390a15b50565b6000546001600160a01b031633146102465760405162461bcd60e51b81526004016101bb90610673565b61021981610483565b6000546001600160a01b031633146102795760405162461bcd60e51b81526004016101bb90610673565b60005b818110156102c5576102b3838383818110610299576102996106a8565b90506020020160208101906102ae91906105d5565b6104ed565b806102bd816106be565b91505061027c565b505050565b6000546001600160a01b031633146102f45760405162461bcd60e51b81526004016101bb90610673565b60005b818110156102c55761032e838383818110610314576103146106a8565b905060200201602081019061032991906105d5565b610483565b80610338816106be565b9150506102f7565b6000546001600160a01b0316331461036a5760405162461bcd60e51b81526004016101bb90610673565b610374600061055c565b565b6000546001600160a01b031633146103a05760405162461bcd60e51b81526004016101bb90610673565b610219816104ed565b60025460009060ff16806103d557506001600160a01b03821660009081526001602052604090205460ff165b156103e35750600019919050565b506000919050565b6000546001600160a01b031633146104155760405162461bcd60e51b81526004016101bb90610673565b6001600160a01b03811661047a5760405162461bcd60e51b815260206004820152602660248201527f4f776e61626c653a206e6577206f776e657220697320746865207a65726f206160448201526564647265737360d01b60648201526084016101bb565b6102198161055c565b6001600160a01b03811660009081526001602052604090205460ff1615610219576001600160a01b038116600081815260016020526040808220805460ff19169055517f0e63d629afe34b3ca5107c10f90abff5091b31551b371758bd50af76834dc0749190a250565b6001600160a01b03811660009081526001602052604090205460ff16610219576001600160a01b0381166000818152600160208190526040808320805460ff1916909217909155517fcd822dc9688e20acea68724a2fbcfe4f3e526d20ecaa37b18fe3047ab377d6a59190a250565b600080546001600160a01b038381166001600160a01b0319831681178455604051919092169283917f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e09190a35050565b6000602082840312156105be57600080fd5b813580151581146105ce57600080fd5b9392505050565b6000602082840312156105e757600080fd5b81356001600160a01b03811681146105ce57600080fd5b6000806020838503121561061157600080fd5b823567ffffffffffffffff8082111561062957600080fd5b818501915085601f83011261063d57600080fd5b81358181111561064c57600080fd5b8660208260051b850101111561066157600080fd5b60209290920196919550909350505050565b6020808252818101527f4f776e61626c653a2063616c6c6572206973206e6f7420746865206f776e6572604082015260600190565b634e487b7160e01b600052603260045260246000fd5b6000600182016106de57634e487b7160e01b600052601160045260246000fd5b506001019056fea264697066735822122014cecb0e463939e23ed1adc0f8ed118c23ed0c50cc32e465823bcba9908f421564736f6c634300080d0033" . parse () . expect ("invalid bytecode")
    });
    pub struct HoprDummyProxyForNetworkRegistry<M>(ethers::contract::Contract<M>);
    impl<M> Clone for HoprDummyProxyForNetworkRegistry<M> {
        fn clone(&self) -> Self {
            HoprDummyProxyForNetworkRegistry(self.0.clone())
        }
    }
    impl<M> std::ops::Deref for HoprDummyProxyForNetworkRegistry<M> {
        type Target = ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> std::fmt::Debug for HoprDummyProxyForNetworkRegistry<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(HoprDummyProxyForNetworkRegistry))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ethers::providers::Middleware> HoprDummyProxyForNetworkRegistry<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            ethers::contract::Contract::new(
                address.into(),
                HOPRDUMMYPROXYFORNETWORKREGISTRY_ABI.clone(),
                client,
            )
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
                HOPRDUMMYPROXYFORNETWORKREGISTRY_ABI.clone(),
                HOPRDUMMYPROXYFORNETWORKREGISTRY_BYTECODE.clone().into(),
                client,
            );
            let deployer = factory.deploy(constructor_args)?;
            let deployer = ethers::contract::ContractDeployer::new(deployer);
            Ok(deployer)
        }
        #[doc = "Calls the contract's `MAX_REGISTRATION_PER_ACCOUNT` (0xf46c84b8) function"]
        pub fn max_registration_per_account(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([244, 108, 132, 184], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `isAllAllowed` (0xf67f5e6f) function"]
        pub fn is_all_allowed(&self) -> ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([246, 127, 94, 111], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `maxAllowedRegistrations` (0xb3544e82) function"]
        pub fn max_allowed_registrations(
            &self,
            account: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([179, 84, 78, 130], account)
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
        #[doc = "Calls the contract's `ownerAddAccount` (0xa71b1b74) function"]
        pub fn owner_add_account(
            &self,
            account: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([167, 27, 27, 116], account)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `ownerBatchAddAccounts` (0x486354e2) function"]
        pub fn owner_batch_add_accounts(
            &self,
            accounts: ::std::vec::Vec<ethers::core::types::Address>,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([72, 99, 84, 226], accounts)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `ownerBatchRemoveAccounts` (0x5c5471e7) function"]
        pub fn owner_batch_remove_accounts(
            &self,
            accounts: ::std::vec::Vec<ethers::core::types::Address>,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([92, 84, 113, 231], accounts)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `ownerRemoveAccount` (0x4051f257) function"]
        pub fn owner_remove_account(
            &self,
            account: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([64, 81, 242, 87], account)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `renounceOwnership` (0x715018a6) function"]
        pub fn renounce_ownership(&self) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([113, 80, 24, 166], ())
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
        #[doc = "Calls the contract's `updateAllowAll` (0x089ffa47) function"]
        pub fn update_allow_all(
            &self,
            updated_allow: bool,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([8, 159, 250, 71], updated_allow)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Gets the contract's `AccountDeregistered` event"]
        pub fn account_deregistered_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, AccountDeregisteredFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `AccountRegistered` event"]
        pub fn account_registered_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, AccountRegisteredFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `AllowAllAccountsEligible` event"]
        pub fn allow_all_accounts_eligible_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, AllowAllAccountsEligibleFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `OwnershipTransferred` event"]
        pub fn ownership_transferred_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, OwnershipTransferredFilter> {
            self.0.event()
        }
        #[doc = r" Returns an [`Event`](#ethers_contract::builders::Event) builder for all events of this contract"]
        pub fn events(
            &self,
        ) -> ethers::contract::builders::Event<M, HoprDummyProxyForNetworkRegistryEvents> {
            self.0.event_with_filter(Default::default())
        }
    }
    impl<M: ethers::providers::Middleware> From<ethers::contract::Contract<M>>
        for HoprDummyProxyForNetworkRegistry<M>
    {
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
    #[ethevent(name = "AccountDeregistered", abi = "AccountDeregistered(address)")]
    pub struct AccountDeregisteredFilter {
        #[ethevent(indexed)]
        pub account: ethers::core::types::Address,
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
    #[ethevent(name = "AccountRegistered", abi = "AccountRegistered(address)")]
    pub struct AccountRegisteredFilter {
        #[ethevent(indexed)]
        pub account: ethers::core::types::Address,
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
        name = "AllowAllAccountsEligible",
        abi = "AllowAllAccountsEligible(bool)"
    )]
    pub struct AllowAllAccountsEligibleFilter {
        pub is_allowed: bool,
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
    #[derive(Debug, Clone, PartialEq, Eq, ethers :: contract :: EthAbiType)]
    pub enum HoprDummyProxyForNetworkRegistryEvents {
        AccountDeregisteredFilter(AccountDeregisteredFilter),
        AccountRegisteredFilter(AccountRegisteredFilter),
        AllowAllAccountsEligibleFilter(AllowAllAccountsEligibleFilter),
        OwnershipTransferredFilter(OwnershipTransferredFilter),
    }
    impl ethers::contract::EthLogDecode for HoprDummyProxyForNetworkRegistryEvents {
        fn decode_log(
            log: &ethers::core::abi::RawLog,
        ) -> ::std::result::Result<Self, ethers::core::abi::Error>
        where
            Self: Sized,
        {
            if let Ok(decoded) = AccountDeregisteredFilter::decode_log(log) {
                return Ok(
                    HoprDummyProxyForNetworkRegistryEvents::AccountDeregisteredFilter(decoded),
                );
            }
            if let Ok(decoded) = AccountRegisteredFilter::decode_log(log) {
                return Ok(HoprDummyProxyForNetworkRegistryEvents::AccountRegisteredFilter(decoded));
            }
            if let Ok(decoded) = AllowAllAccountsEligibleFilter::decode_log(log) {
                return Ok(
                    HoprDummyProxyForNetworkRegistryEvents::AllowAllAccountsEligibleFilter(decoded),
                );
            }
            if let Ok(decoded) = OwnershipTransferredFilter::decode_log(log) {
                return Ok(
                    HoprDummyProxyForNetworkRegistryEvents::OwnershipTransferredFilter(decoded),
                );
            }
            Err(ethers::core::abi::Error::InvalidData)
        }
    }
    impl ::std::fmt::Display for HoprDummyProxyForNetworkRegistryEvents {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                HoprDummyProxyForNetworkRegistryEvents::AccountDeregisteredFilter(element) => {
                    element.fmt(f)
                }
                HoprDummyProxyForNetworkRegistryEvents::AccountRegisteredFilter(element) => {
                    element.fmt(f)
                }
                HoprDummyProxyForNetworkRegistryEvents::AllowAllAccountsEligibleFilter(element) => {
                    element.fmt(f)
                }
                HoprDummyProxyForNetworkRegistryEvents::OwnershipTransferredFilter(element) => {
                    element.fmt(f)
                }
            }
        }
    }
    #[doc = "Container type for all input parameters for the `MAX_REGISTRATION_PER_ACCOUNT` function with signature `MAX_REGISTRATION_PER_ACCOUNT()` and selector `[244, 108, 132, 184]`"]
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
        name = "MAX_REGISTRATION_PER_ACCOUNT",
        abi = "MAX_REGISTRATION_PER_ACCOUNT()"
    )]
    pub struct MaxRegistrationPerAccountCall;
    #[doc = "Container type for all input parameters for the `isAllAllowed` function with signature `isAllAllowed()` and selector `[246, 127, 94, 111]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "isAllAllowed", abi = "isAllAllowed()")]
    pub struct IsAllAllowedCall;
    #[doc = "Container type for all input parameters for the `maxAllowedRegistrations` function with signature `maxAllowedRegistrations(address)` and selector `[179, 84, 78, 130]`"]
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
        name = "maxAllowedRegistrations",
        abi = "maxAllowedRegistrations(address)"
    )]
    pub struct MaxAllowedRegistrationsCall {
        pub account: ethers::core::types::Address,
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
    #[doc = "Container type for all input parameters for the `ownerAddAccount` function with signature `ownerAddAccount(address)` and selector `[167, 27, 27, 116]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "ownerAddAccount", abi = "ownerAddAccount(address)")]
    pub struct OwnerAddAccountCall {
        pub account: ethers::core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `ownerBatchAddAccounts` function with signature `ownerBatchAddAccounts(address[])` and selector `[72, 99, 84, 226]`"]
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
        name = "ownerBatchAddAccounts",
        abi = "ownerBatchAddAccounts(address[])"
    )]
    pub struct OwnerBatchAddAccountsCall {
        pub accounts: ::std::vec::Vec<ethers::core::types::Address>,
    }
    #[doc = "Container type for all input parameters for the `ownerBatchRemoveAccounts` function with signature `ownerBatchRemoveAccounts(address[])` and selector `[92, 84, 113, 231]`"]
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
        name = "ownerBatchRemoveAccounts",
        abi = "ownerBatchRemoveAccounts(address[])"
    )]
    pub struct OwnerBatchRemoveAccountsCall {
        pub accounts: ::std::vec::Vec<ethers::core::types::Address>,
    }
    #[doc = "Container type for all input parameters for the `ownerRemoveAccount` function with signature `ownerRemoveAccount(address)` and selector `[64, 81, 242, 87]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "ownerRemoveAccount", abi = "ownerRemoveAccount(address)")]
    pub struct OwnerRemoveAccountCall {
        pub account: ethers::core::types::Address,
    }
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
    #[doc = "Container type for all input parameters for the `updateAllowAll` function with signature `updateAllowAll(bool)` and selector `[8, 159, 250, 71]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "updateAllowAll", abi = "updateAllowAll(bool)")]
    pub struct UpdateAllowAllCall {
        pub updated_allow: bool,
    }
    #[derive(Debug, Clone, PartialEq, Eq, ethers :: contract :: EthAbiType)]
    pub enum HoprDummyProxyForNetworkRegistryCalls {
        MaxRegistrationPerAccount(MaxRegistrationPerAccountCall),
        IsAllAllowed(IsAllAllowedCall),
        MaxAllowedRegistrations(MaxAllowedRegistrationsCall),
        Owner(OwnerCall),
        OwnerAddAccount(OwnerAddAccountCall),
        OwnerBatchAddAccounts(OwnerBatchAddAccountsCall),
        OwnerBatchRemoveAccounts(OwnerBatchRemoveAccountsCall),
        OwnerRemoveAccount(OwnerRemoveAccountCall),
        RenounceOwnership(RenounceOwnershipCall),
        TransferOwnership(TransferOwnershipCall),
        UpdateAllowAll(UpdateAllowAllCall),
    }
    impl ethers::core::abi::AbiDecode for HoprDummyProxyForNetworkRegistryCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::std::result::Result<Self, ethers::core::abi::AbiError> {
            if let Ok(decoded) =
                <MaxRegistrationPerAccountCall as ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                )
            {
                return Ok(
                    HoprDummyProxyForNetworkRegistryCalls::MaxRegistrationPerAccount(decoded),
                );
            }
            if let Ok(decoded) =
                <IsAllAllowedCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprDummyProxyForNetworkRegistryCalls::IsAllAllowed(decoded));
            }
            if let Ok(decoded) =
                <MaxAllowedRegistrationsCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprDummyProxyForNetworkRegistryCalls::MaxAllowedRegistrations(decoded));
            }
            if let Ok(decoded) = <OwnerCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprDummyProxyForNetworkRegistryCalls::Owner(decoded));
            }
            if let Ok(decoded) =
                <OwnerAddAccountCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprDummyProxyForNetworkRegistryCalls::OwnerAddAccount(
                    decoded,
                ));
            }
            if let Ok(decoded) =
                <OwnerBatchAddAccountsCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprDummyProxyForNetworkRegistryCalls::OwnerBatchAddAccounts(decoded));
            }
            if let Ok(decoded) =
                <OwnerBatchRemoveAccountsCall as ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                )
            {
                return Ok(HoprDummyProxyForNetworkRegistryCalls::OwnerBatchRemoveAccounts(decoded));
            }
            if let Ok(decoded) =
                <OwnerRemoveAccountCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprDummyProxyForNetworkRegistryCalls::OwnerRemoveAccount(
                    decoded,
                ));
            }
            if let Ok(decoded) =
                <RenounceOwnershipCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprDummyProxyForNetworkRegistryCalls::RenounceOwnership(
                    decoded,
                ));
            }
            if let Ok(decoded) =
                <TransferOwnershipCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprDummyProxyForNetworkRegistryCalls::TransferOwnership(
                    decoded,
                ));
            }
            if let Ok(decoded) =
                <UpdateAllowAllCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprDummyProxyForNetworkRegistryCalls::UpdateAllowAll(
                    decoded,
                ));
            }
            Err(ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ethers::core::abi::AbiEncode for HoprDummyProxyForNetworkRegistryCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                HoprDummyProxyForNetworkRegistryCalls::MaxRegistrationPerAccount(element) => {
                    element.encode()
                }
                HoprDummyProxyForNetworkRegistryCalls::IsAllAllowed(element) => element.encode(),
                HoprDummyProxyForNetworkRegistryCalls::MaxAllowedRegistrations(element) => {
                    element.encode()
                }
                HoprDummyProxyForNetworkRegistryCalls::Owner(element) => element.encode(),
                HoprDummyProxyForNetworkRegistryCalls::OwnerAddAccount(element) => element.encode(),
                HoprDummyProxyForNetworkRegistryCalls::OwnerBatchAddAccounts(element) => {
                    element.encode()
                }
                HoprDummyProxyForNetworkRegistryCalls::OwnerBatchRemoveAccounts(element) => {
                    element.encode()
                }
                HoprDummyProxyForNetworkRegistryCalls::OwnerRemoveAccount(element) => {
                    element.encode()
                }
                HoprDummyProxyForNetworkRegistryCalls::RenounceOwnership(element) => {
                    element.encode()
                }
                HoprDummyProxyForNetworkRegistryCalls::TransferOwnership(element) => {
                    element.encode()
                }
                HoprDummyProxyForNetworkRegistryCalls::UpdateAllowAll(element) => element.encode(),
            }
        }
    }
    impl ::std::fmt::Display for HoprDummyProxyForNetworkRegistryCalls {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                HoprDummyProxyForNetworkRegistryCalls::MaxRegistrationPerAccount(element) => {
                    element.fmt(f)
                }
                HoprDummyProxyForNetworkRegistryCalls::IsAllAllowed(element) => element.fmt(f),
                HoprDummyProxyForNetworkRegistryCalls::MaxAllowedRegistrations(element) => {
                    element.fmt(f)
                }
                HoprDummyProxyForNetworkRegistryCalls::Owner(element) => element.fmt(f),
                HoprDummyProxyForNetworkRegistryCalls::OwnerAddAccount(element) => element.fmt(f),
                HoprDummyProxyForNetworkRegistryCalls::OwnerBatchAddAccounts(element) => {
                    element.fmt(f)
                }
                HoprDummyProxyForNetworkRegistryCalls::OwnerBatchRemoveAccounts(element) => {
                    element.fmt(f)
                }
                HoprDummyProxyForNetworkRegistryCalls::OwnerRemoveAccount(element) => {
                    element.fmt(f)
                }
                HoprDummyProxyForNetworkRegistryCalls::RenounceOwnership(element) => element.fmt(f),
                HoprDummyProxyForNetworkRegistryCalls::TransferOwnership(element) => element.fmt(f),
                HoprDummyProxyForNetworkRegistryCalls::UpdateAllowAll(element) => element.fmt(f),
            }
        }
    }
    impl ::std::convert::From<MaxRegistrationPerAccountCall> for HoprDummyProxyForNetworkRegistryCalls {
        fn from(var: MaxRegistrationPerAccountCall) -> Self {
            HoprDummyProxyForNetworkRegistryCalls::MaxRegistrationPerAccount(var)
        }
    }
    impl ::std::convert::From<IsAllAllowedCall> for HoprDummyProxyForNetworkRegistryCalls {
        fn from(var: IsAllAllowedCall) -> Self {
            HoprDummyProxyForNetworkRegistryCalls::IsAllAllowed(var)
        }
    }
    impl ::std::convert::From<MaxAllowedRegistrationsCall> for HoprDummyProxyForNetworkRegistryCalls {
        fn from(var: MaxAllowedRegistrationsCall) -> Self {
            HoprDummyProxyForNetworkRegistryCalls::MaxAllowedRegistrations(var)
        }
    }
    impl ::std::convert::From<OwnerCall> for HoprDummyProxyForNetworkRegistryCalls {
        fn from(var: OwnerCall) -> Self {
            HoprDummyProxyForNetworkRegistryCalls::Owner(var)
        }
    }
    impl ::std::convert::From<OwnerAddAccountCall> for HoprDummyProxyForNetworkRegistryCalls {
        fn from(var: OwnerAddAccountCall) -> Self {
            HoprDummyProxyForNetworkRegistryCalls::OwnerAddAccount(var)
        }
    }
    impl ::std::convert::From<OwnerBatchAddAccountsCall> for HoprDummyProxyForNetworkRegistryCalls {
        fn from(var: OwnerBatchAddAccountsCall) -> Self {
            HoprDummyProxyForNetworkRegistryCalls::OwnerBatchAddAccounts(var)
        }
    }
    impl ::std::convert::From<OwnerBatchRemoveAccountsCall> for HoprDummyProxyForNetworkRegistryCalls {
        fn from(var: OwnerBatchRemoveAccountsCall) -> Self {
            HoprDummyProxyForNetworkRegistryCalls::OwnerBatchRemoveAccounts(var)
        }
    }
    impl ::std::convert::From<OwnerRemoveAccountCall> for HoprDummyProxyForNetworkRegistryCalls {
        fn from(var: OwnerRemoveAccountCall) -> Self {
            HoprDummyProxyForNetworkRegistryCalls::OwnerRemoveAccount(var)
        }
    }
    impl ::std::convert::From<RenounceOwnershipCall> for HoprDummyProxyForNetworkRegistryCalls {
        fn from(var: RenounceOwnershipCall) -> Self {
            HoprDummyProxyForNetworkRegistryCalls::RenounceOwnership(var)
        }
    }
    impl ::std::convert::From<TransferOwnershipCall> for HoprDummyProxyForNetworkRegistryCalls {
        fn from(var: TransferOwnershipCall) -> Self {
            HoprDummyProxyForNetworkRegistryCalls::TransferOwnership(var)
        }
    }
    impl ::std::convert::From<UpdateAllowAllCall> for HoprDummyProxyForNetworkRegistryCalls {
        fn from(var: UpdateAllowAllCall) -> Self {
            HoprDummyProxyForNetworkRegistryCalls::UpdateAllowAll(var)
        }
    }
    #[doc = "Container type for all return fields from the `MAX_REGISTRATION_PER_ACCOUNT` function with signature `MAX_REGISTRATION_PER_ACCOUNT()` and selector `[244, 108, 132, 184]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct MaxRegistrationPerAccountReturn(pub ethers::core::types::U256);
    #[doc = "Container type for all return fields from the `isAllAllowed` function with signature `isAllAllowed()` and selector `[246, 127, 94, 111]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct IsAllAllowedReturn(pub bool);
    #[doc = "Container type for all return fields from the `maxAllowedRegistrations` function with signature `maxAllowedRegistrations(address)` and selector `[179, 84, 78, 130]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct MaxAllowedRegistrationsReturn(pub ethers::core::types::U256);
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
}
