pub use ierc1820_registry::*;
#[allow(clippy::too_many_arguments, non_camel_case_types)]
pub mod ierc1820_registry {
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
    #[doc = "IERC1820Registry was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    # [rustfmt :: skip] const __ABI : & str = "[{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"bytes32\",\"name\":\"interfaceHash\",\"type\":\"bytes32\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"implementer\",\"type\":\"address\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"InterfaceImplementerSet\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"newManager\",\"type\":\"address\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"ManagerChanged\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"_interfaceHash\",\"type\":\"bytes32\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"getInterfaceImplementer\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"getManager\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"bytes4\",\"name\":\"interfaceId\",\"type\":\"bytes4\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"implementsERC165Interface\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"bytes4\",\"name\":\"interfaceId\",\"type\":\"bytes4\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"implementsERC165InterfaceNoCache\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"interfaceName\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"pure\",\"type\":\"function\",\"name\":\"interfaceHash\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"_interfaceHash\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"implementer\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"setInterfaceImplementer\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"newManager\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"setManager\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"bytes4\",\"name\":\"interfaceId\",\"type\":\"bytes4\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"updateERC165Cache\",\"outputs\":[]}]" ;
    #[doc = r" The parsed JSON-ABI of the contract."]
    pub static IERC1820REGISTRY_ABI: ethers::contract::Lazy<ethers::core::abi::Abi> =
        ethers::contract::Lazy::new(|| {
            ethers::core::utils::__serde_json::from_str(__ABI).expect("invalid abi")
        });
    pub struct IERC1820Registry<M>(ethers::contract::Contract<M>);
    impl<M> Clone for IERC1820Registry<M> {
        fn clone(&self) -> Self {
            IERC1820Registry(self.0.clone())
        }
    }
    impl<M> std::ops::Deref for IERC1820Registry<M> {
        type Target = ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> std::fmt::Debug for IERC1820Registry<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(IERC1820Registry))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ethers::providers::Middleware> IERC1820Registry<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            ethers::contract::Contract::new(address.into(), IERC1820REGISTRY_ABI.clone(), client)
                .into()
        }
        #[doc = "Calls the contract's `getInterfaceImplementer` (0xaabbb8ca) function"]
        pub fn get_interface_implementer(
            &self,
            account: ethers::core::types::Address,
            interface_hash: [u8; 32],
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Address> {
            self.0
                .method_hash([170, 187, 184, 202], (account, interface_hash))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `getManager` (0x3d584063) function"]
        pub fn get_manager(
            &self,
            account: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Address> {
            self.0
                .method_hash([61, 88, 64, 99], account)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `implementsERC165Interface` (0xf712f3e8) function"]
        pub fn implements_erc165_interface(
            &self,
            account: ethers::core::types::Address,
            interface_id: [u8; 4],
        ) -> ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([247, 18, 243, 232], (account, interface_id))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `implementsERC165InterfaceNoCache` (0xb7056765) function"]
        pub fn implements_erc165_interface_no_cache(
            &self,
            account: ethers::core::types::Address,
            interface_id: [u8; 4],
        ) -> ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([183, 5, 103, 101], (account, interface_id))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `interfaceHash` (0x65ba36c1) function"]
        pub fn interface_hash(
            &self,
            interface_name: String,
        ) -> ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([101, 186, 54, 193], interface_name)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `setInterfaceImplementer` (0x29965a1d) function"]
        pub fn set_interface_implementer(
            &self,
            account: ethers::core::types::Address,
            interface_hash: [u8; 32],
            implementer: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([41, 150, 90, 29], (account, interface_hash, implementer))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `setManager` (0x5df8122f) function"]
        pub fn set_manager(
            &self,
            account: ethers::core::types::Address,
            new_manager: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([93, 248, 18, 47], (account, new_manager))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `updateERC165Cache` (0xa41e7d51) function"]
        pub fn update_erc165_cache(
            &self,
            account: ethers::core::types::Address,
            interface_id: [u8; 4],
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([164, 30, 125, 81], (account, interface_id))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Gets the contract's `InterfaceImplementerSet` event"]
        pub fn interface_implementer_set_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, InterfaceImplementerSetFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `ManagerChanged` event"]
        pub fn manager_changed_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, ManagerChangedFilter> {
            self.0.event()
        }
        #[doc = r" Returns an [`Event`](#ethers_contract::builders::Event) builder for all events of this contract"]
        pub fn events(&self) -> ethers::contract::builders::Event<M, IERC1820RegistryEvents> {
            self.0.event_with_filter(Default::default())
        }
    }
    impl<M: ethers::providers::Middleware> From<ethers::contract::Contract<M>> for IERC1820Registry<M> {
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
        name = "InterfaceImplementerSet",
        abi = "InterfaceImplementerSet(address,bytes32,address)"
    )]
    pub struct InterfaceImplementerSetFilter {
        #[ethevent(indexed)]
        pub account: ethers::core::types::Address,
        #[ethevent(indexed)]
        pub interface_hash: [u8; 32],
        #[ethevent(indexed)]
        pub implementer: ethers::core::types::Address,
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
    #[ethevent(name = "ManagerChanged", abi = "ManagerChanged(address,address)")]
    pub struct ManagerChangedFilter {
        #[ethevent(indexed)]
        pub account: ethers::core::types::Address,
        #[ethevent(indexed)]
        pub new_manager: ethers::core::types::Address,
    }
    #[derive(Debug, Clone, PartialEq, Eq, ethers :: contract :: EthAbiType)]
    pub enum IERC1820RegistryEvents {
        InterfaceImplementerSetFilter(InterfaceImplementerSetFilter),
        ManagerChangedFilter(ManagerChangedFilter),
    }
    impl ethers::contract::EthLogDecode for IERC1820RegistryEvents {
        fn decode_log(
            log: &ethers::core::abi::RawLog,
        ) -> ::std::result::Result<Self, ethers::core::abi::Error>
        where
            Self: Sized,
        {
            if let Ok(decoded) = InterfaceImplementerSetFilter::decode_log(log) {
                return Ok(IERC1820RegistryEvents::InterfaceImplementerSetFilter(
                    decoded,
                ));
            }
            if let Ok(decoded) = ManagerChangedFilter::decode_log(log) {
                return Ok(IERC1820RegistryEvents::ManagerChangedFilter(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData)
        }
    }
    impl ::std::fmt::Display for IERC1820RegistryEvents {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                IERC1820RegistryEvents::InterfaceImplementerSetFilter(element) => element.fmt(f),
                IERC1820RegistryEvents::ManagerChangedFilter(element) => element.fmt(f),
            }
        }
    }
    #[doc = "Container type for all input parameters for the `getInterfaceImplementer` function with signature `getInterfaceImplementer(address,bytes32)` and selector `[170, 187, 184, 202]`"]
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
        name = "getInterfaceImplementer",
        abi = "getInterfaceImplementer(address,bytes32)"
    )]
    pub struct GetInterfaceImplementerCall {
        pub account: ethers::core::types::Address,
        pub interface_hash: [u8; 32],
    }
    #[doc = "Container type for all input parameters for the `getManager` function with signature `getManager(address)` and selector `[61, 88, 64, 99]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "getManager", abi = "getManager(address)")]
    pub struct GetManagerCall {
        pub account: ethers::core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `implementsERC165Interface` function with signature `implementsERC165Interface(address,bytes4)` and selector `[247, 18, 243, 232]`"]
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
        name = "implementsERC165Interface",
        abi = "implementsERC165Interface(address,bytes4)"
    )]
    pub struct ImplementsERC165InterfaceCall {
        pub account: ethers::core::types::Address,
        pub interface_id: [u8; 4],
    }
    #[doc = "Container type for all input parameters for the `implementsERC165InterfaceNoCache` function with signature `implementsERC165InterfaceNoCache(address,bytes4)` and selector `[183, 5, 103, 101]`"]
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
        name = "implementsERC165InterfaceNoCache",
        abi = "implementsERC165InterfaceNoCache(address,bytes4)"
    )]
    pub struct ImplementsERC165InterfaceNoCacheCall {
        pub account: ethers::core::types::Address,
        pub interface_id: [u8; 4],
    }
    #[doc = "Container type for all input parameters for the `interfaceHash` function with signature `interfaceHash(string)` and selector `[101, 186, 54, 193]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "interfaceHash", abi = "interfaceHash(string)")]
    pub struct InterfaceHashCall {
        pub interface_name: String,
    }
    #[doc = "Container type for all input parameters for the `setInterfaceImplementer` function with signature `setInterfaceImplementer(address,bytes32,address)` and selector `[41, 150, 90, 29]`"]
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
        name = "setInterfaceImplementer",
        abi = "setInterfaceImplementer(address,bytes32,address)"
    )]
    pub struct SetInterfaceImplementerCall {
        pub account: ethers::core::types::Address,
        pub interface_hash: [u8; 32],
        pub implementer: ethers::core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `setManager` function with signature `setManager(address,address)` and selector `[93, 248, 18, 47]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "setManager", abi = "setManager(address,address)")]
    pub struct SetManagerCall {
        pub account: ethers::core::types::Address,
        pub new_manager: ethers::core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `updateERC165Cache` function with signature `updateERC165Cache(address,bytes4)` and selector `[164, 30, 125, 81]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "updateERC165Cache", abi = "updateERC165Cache(address,bytes4)")]
    pub struct UpdateERC165CacheCall {
        pub account: ethers::core::types::Address,
        pub interface_id: [u8; 4],
    }
    #[derive(Debug, Clone, PartialEq, Eq, ethers :: contract :: EthAbiType)]
    pub enum IERC1820RegistryCalls {
        GetInterfaceImplementer(GetInterfaceImplementerCall),
        GetManager(GetManagerCall),
        ImplementsERC165Interface(ImplementsERC165InterfaceCall),
        ImplementsERC165InterfaceNoCache(ImplementsERC165InterfaceNoCacheCall),
        InterfaceHash(InterfaceHashCall),
        SetInterfaceImplementer(SetInterfaceImplementerCall),
        SetManager(SetManagerCall),
        UpdateERC165Cache(UpdateERC165CacheCall),
    }
    impl ethers::core::abi::AbiDecode for IERC1820RegistryCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::std::result::Result<Self, ethers::core::abi::AbiError> {
            if let Ok(decoded) =
                <GetInterfaceImplementerCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IERC1820RegistryCalls::GetInterfaceImplementer(decoded));
            }
            if let Ok(decoded) =
                <GetManagerCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IERC1820RegistryCalls::GetManager(decoded));
            }
            if let Ok(decoded) =
                <ImplementsERC165InterfaceCall as ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                )
            {
                return Ok(IERC1820RegistryCalls::ImplementsERC165Interface(decoded));
            }
            if let Ok(decoded) =
                <ImplementsERC165InterfaceNoCacheCall as ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                )
            {
                return Ok(IERC1820RegistryCalls::ImplementsERC165InterfaceNoCache(
                    decoded,
                ));
            }
            if let Ok(decoded) =
                <InterfaceHashCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IERC1820RegistryCalls::InterfaceHash(decoded));
            }
            if let Ok(decoded) =
                <SetInterfaceImplementerCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IERC1820RegistryCalls::SetInterfaceImplementer(decoded));
            }
            if let Ok(decoded) =
                <SetManagerCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IERC1820RegistryCalls::SetManager(decoded));
            }
            if let Ok(decoded) =
                <UpdateERC165CacheCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IERC1820RegistryCalls::UpdateERC165Cache(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ethers::core::abi::AbiEncode for IERC1820RegistryCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                IERC1820RegistryCalls::GetInterfaceImplementer(element) => element.encode(),
                IERC1820RegistryCalls::GetManager(element) => element.encode(),
                IERC1820RegistryCalls::ImplementsERC165Interface(element) => element.encode(),
                IERC1820RegistryCalls::ImplementsERC165InterfaceNoCache(element) => {
                    element.encode()
                }
                IERC1820RegistryCalls::InterfaceHash(element) => element.encode(),
                IERC1820RegistryCalls::SetInterfaceImplementer(element) => element.encode(),
                IERC1820RegistryCalls::SetManager(element) => element.encode(),
                IERC1820RegistryCalls::UpdateERC165Cache(element) => element.encode(),
            }
        }
    }
    impl ::std::fmt::Display for IERC1820RegistryCalls {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                IERC1820RegistryCalls::GetInterfaceImplementer(element) => element.fmt(f),
                IERC1820RegistryCalls::GetManager(element) => element.fmt(f),
                IERC1820RegistryCalls::ImplementsERC165Interface(element) => element.fmt(f),
                IERC1820RegistryCalls::ImplementsERC165InterfaceNoCache(element) => element.fmt(f),
                IERC1820RegistryCalls::InterfaceHash(element) => element.fmt(f),
                IERC1820RegistryCalls::SetInterfaceImplementer(element) => element.fmt(f),
                IERC1820RegistryCalls::SetManager(element) => element.fmt(f),
                IERC1820RegistryCalls::UpdateERC165Cache(element) => element.fmt(f),
            }
        }
    }
    impl ::std::convert::From<GetInterfaceImplementerCall> for IERC1820RegistryCalls {
        fn from(var: GetInterfaceImplementerCall) -> Self {
            IERC1820RegistryCalls::GetInterfaceImplementer(var)
        }
    }
    impl ::std::convert::From<GetManagerCall> for IERC1820RegistryCalls {
        fn from(var: GetManagerCall) -> Self {
            IERC1820RegistryCalls::GetManager(var)
        }
    }
    impl ::std::convert::From<ImplementsERC165InterfaceCall> for IERC1820RegistryCalls {
        fn from(var: ImplementsERC165InterfaceCall) -> Self {
            IERC1820RegistryCalls::ImplementsERC165Interface(var)
        }
    }
    impl ::std::convert::From<ImplementsERC165InterfaceNoCacheCall> for IERC1820RegistryCalls {
        fn from(var: ImplementsERC165InterfaceNoCacheCall) -> Self {
            IERC1820RegistryCalls::ImplementsERC165InterfaceNoCache(var)
        }
    }
    impl ::std::convert::From<InterfaceHashCall> for IERC1820RegistryCalls {
        fn from(var: InterfaceHashCall) -> Self {
            IERC1820RegistryCalls::InterfaceHash(var)
        }
    }
    impl ::std::convert::From<SetInterfaceImplementerCall> for IERC1820RegistryCalls {
        fn from(var: SetInterfaceImplementerCall) -> Self {
            IERC1820RegistryCalls::SetInterfaceImplementer(var)
        }
    }
    impl ::std::convert::From<SetManagerCall> for IERC1820RegistryCalls {
        fn from(var: SetManagerCall) -> Self {
            IERC1820RegistryCalls::SetManager(var)
        }
    }
    impl ::std::convert::From<UpdateERC165CacheCall> for IERC1820RegistryCalls {
        fn from(var: UpdateERC165CacheCall) -> Self {
            IERC1820RegistryCalls::UpdateERC165Cache(var)
        }
    }
    #[doc = "Container type for all return fields from the `getInterfaceImplementer` function with signature `getInterfaceImplementer(address,bytes32)` and selector `[170, 187, 184, 202]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct GetInterfaceImplementerReturn(pub ethers::core::types::Address);
    #[doc = "Container type for all return fields from the `getManager` function with signature `getManager(address)` and selector `[61, 88, 64, 99]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct GetManagerReturn(pub ethers::core::types::Address);
    #[doc = "Container type for all return fields from the `implementsERC165Interface` function with signature `implementsERC165Interface(address,bytes4)` and selector `[247, 18, 243, 232]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct ImplementsERC165InterfaceReturn(pub bool);
    #[doc = "Container type for all return fields from the `implementsERC165InterfaceNoCache` function with signature `implementsERC165InterfaceNoCache(address,bytes4)` and selector `[183, 5, 103, 101]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct ImplementsERC165InterfaceNoCacheReturn(pub bool);
    #[doc = "Container type for all return fields from the `interfaceHash` function with signature `interfaceHash(string)` and selector `[101, 186, 54, 193]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct InterfaceHashReturn(pub [u8; 32]);
}
