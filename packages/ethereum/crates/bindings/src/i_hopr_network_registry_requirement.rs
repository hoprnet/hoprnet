pub use i_hopr_network_registry_requirement::*;
#[allow(clippy::too_many_arguments, non_camel_case_types)]
pub mod i_hopr_network_registry_requirement {
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
    #[doc = "IHoprNetworkRegistryRequirement was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    # [rustfmt :: skip] const __ABI : & str = "[{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"maxAllowedRegistrations\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]}]" ;
    #[doc = r" The parsed JSON-ABI of the contract."]
    pub static IHOPRNETWORKREGISTRYREQUIREMENT_ABI: ethers::contract::Lazy<ethers::core::abi::Abi> =
        ethers::contract::Lazy::new(|| {
            ethers::core::utils::__serde_json::from_str(__ABI).expect("invalid abi")
        });
    pub struct IHoprNetworkRegistryRequirement<M>(ethers::contract::Contract<M>);
    impl<M> Clone for IHoprNetworkRegistryRequirement<M> {
        fn clone(&self) -> Self {
            IHoprNetworkRegistryRequirement(self.0.clone())
        }
    }
    impl<M> std::ops::Deref for IHoprNetworkRegistryRequirement<M> {
        type Target = ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> std::fmt::Debug for IHoprNetworkRegistryRequirement<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(IHoprNetworkRegistryRequirement))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ethers::providers::Middleware> IHoprNetworkRegistryRequirement<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            ethers::contract::Contract::new(
                address.into(),
                IHOPRNETWORKREGISTRYREQUIREMENT_ABI.clone(),
                client,
            )
            .into()
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
    }
    impl<M: ethers::providers::Middleware> From<ethers::contract::Contract<M>>
        for IHoprNetworkRegistryRequirement<M>
    {
        fn from(contract: ethers::contract::Contract<M>) -> Self {
            Self(contract)
        }
    }
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
}
