pub use ierc1820_implementer::*;
#[allow(clippy::too_many_arguments, non_camel_case_types)]
pub mod ierc1820_implementer {
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
    #[doc = "IERC1820Implementer was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    # [rustfmt :: skip] const __ABI : & str = "[{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"interfaceHash\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"canImplementInterfaceForAddress\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[]}]}]" ;
    #[doc = r" The parsed JSON-ABI of the contract."]
    pub static IERC1820IMPLEMENTER_ABI: ethers::contract::Lazy<ethers::core::abi::Abi> =
        ethers::contract::Lazy::new(|| {
            ethers::core::utils::__serde_json::from_str(__ABI).expect("invalid abi")
        });
    pub struct IERC1820Implementer<M>(ethers::contract::Contract<M>);
    impl<M> Clone for IERC1820Implementer<M> {
        fn clone(&self) -> Self {
            IERC1820Implementer(self.0.clone())
        }
    }
    impl<M> std::ops::Deref for IERC1820Implementer<M> {
        type Target = ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> std::fmt::Debug for IERC1820Implementer<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(IERC1820Implementer))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ethers::providers::Middleware> IERC1820Implementer<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            ethers::contract::Contract::new(address.into(), IERC1820IMPLEMENTER_ABI.clone(), client)
                .into()
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
    }
    impl<M: ethers::providers::Middleware> From<ethers::contract::Contract<M>>
        for IERC1820Implementer<M>
    {
        fn from(contract: ethers::contract::Contract<M>) -> Self {
            Self(contract)
        }
    }
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
}
