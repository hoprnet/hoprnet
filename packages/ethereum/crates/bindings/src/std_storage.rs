pub use std_storage::*;
#[allow(clippy::too_many_arguments, non_camel_case_types)]
pub mod std_storage {
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
    #[doc = "stdStorage was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    # [rustfmt :: skip] const __ABI : & str = "[{\"inputs\":[{\"internalType\":\"address\",\"name\":\"who\",\"type\":\"address\",\"components\":[],\"indexed\":false},{\"internalType\":\"bytes4\",\"name\":\"fsig\",\"type\":\"bytes4\",\"components\":[],\"indexed\":false},{\"internalType\":\"bytes32\",\"name\":\"keysHash\",\"type\":\"bytes32\",\"components\":[],\"indexed\":false},{\"internalType\":\"uint256\",\"name\":\"slot\",\"type\":\"uint256\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"SlotFound\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"who\",\"type\":\"address\",\"components\":[],\"indexed\":false},{\"internalType\":\"uint256\",\"name\":\"slot\",\"type\":\"uint256\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"WARNING_UninitedSlot\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"bytes\",\"name\":\"b\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"offset\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"pure\",\"type\":\"function\",\"name\":\"bytesToBytes32\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[]}]}]" ;
    #[doc = r" The parsed JSON-ABI of the contract."]
    pub static STDSTORAGE_ABI: ethers::contract::Lazy<ethers::core::abi::Abi> =
        ethers::contract::Lazy::new(|| {
            ethers::core::utils::__serde_json::from_str(__ABI).expect("invalid abi")
        });
    #[doc = r" Bytecode of the #name contract"]
    pub static STDSTORAGE_BYTECODE: ethers::contract::Lazy<ethers::core::types::Bytes> =
        ethers::contract::Lazy::new(|| {
            "0x61025961003a600b82828239805160001a60731461002d57634e487b7160e01b600052600060045260246000fd5b30600052607381538281f3fe73000000000000000000000000000000000000000030146080604052600436106100355760003560e01c8063535849391461003a575b600080fd5b61004d6100483660046100f2565b61005f565b60405190815260200160405180910390f35b60008060006020855111610074578451610077565b60205b905060005b818110156100d25761008f8160086101bd565b8661009a83886101dc565b815181106100aa576100aa6101f4565b01602001516001600160f81b031916901c9290921791806100ca8161020a565b91505061007c565b5090949350505050565b634e487b7160e01b600052604160045260246000fd5b6000806040838503121561010557600080fd5b823567ffffffffffffffff8082111561011d57600080fd5b818501915085601f83011261013157600080fd5b813581811115610143576101436100dc565b604051601f8201601f19908116603f0116810190838211818310171561016b5761016b6100dc565b8160405282815288602084870101111561018457600080fd5b826020860160208301376000602093820184015298969091013596505050505050565b634e487b7160e01b600052601160045260246000fd5b60008160001904831182151516156101d7576101d76101a7565b500290565b600082198211156101ef576101ef6101a7565b500190565b634e487b7160e01b600052603260045260246000fd5b60006001820161021c5761021c6101a7565b506001019056fea2646970667358221220aac2de7f9790a343c914ba8859ac95c5e9ef02b957a5ad25bf444437f6fe069664736f6c634300080d0033" . parse () . expect ("invalid bytecode")
        });
    pub struct stdStorage<M>(ethers::contract::Contract<M>);
    impl<M> Clone for stdStorage<M> {
        fn clone(&self) -> Self {
            stdStorage(self.0.clone())
        }
    }
    impl<M> std::ops::Deref for stdStorage<M> {
        type Target = ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> std::fmt::Debug for stdStorage<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(stdStorage))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ethers::providers::Middleware> stdStorage<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            ethers::contract::Contract::new(address.into(), STDSTORAGE_ABI.clone(), client).into()
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
                STDSTORAGE_ABI.clone(),
                STDSTORAGE_BYTECODE.clone().into(),
                client,
            );
            let deployer = factory.deploy(constructor_args)?;
            let deployer = ethers::contract::ContractDeployer::new(deployer);
            Ok(deployer)
        }
        #[doc = "Calls the contract's `bytesToBytes32` (0x53584939) function"]
        pub fn bytes_to_bytes_32(
            &self,
            b: ethers::core::types::Bytes,
            offset: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([83, 88, 73, 57], (b, offset))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Gets the contract's `SlotFound` event"]
        pub fn slot_found_filter(&self) -> ethers::contract::builders::Event<M, SlotFoundFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `WARNING_UninitedSlot` event"]
        pub fn warning_uninited_slot_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, WarningUninitedSlotFilter> {
            self.0.event()
        }
        #[doc = r" Returns an [`Event`](#ethers_contract::builders::Event) builder for all events of this contract"]
        pub fn events(&self) -> ethers::contract::builders::Event<M, stdStorageEvents> {
            self.0.event_with_filter(Default::default())
        }
    }
    impl<M: ethers::providers::Middleware> From<ethers::contract::Contract<M>> for stdStorage<M> {
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
    #[ethevent(name = "SlotFound", abi = "SlotFound(address,bytes4,bytes32,uint256)")]
    pub struct SlotFoundFilter {
        pub who: ethers::core::types::Address,
        pub fsig: [u8; 4],
        pub keys_hash: [u8; 32],
        pub slot: ethers::core::types::U256,
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
        name = "WARNING_UninitedSlot",
        abi = "WARNING_UninitedSlot(address,uint256)"
    )]
    pub struct WarningUninitedSlotFilter {
        pub who: ethers::core::types::Address,
        pub slot: ethers::core::types::U256,
    }
    #[derive(Debug, Clone, PartialEq, Eq, ethers :: contract :: EthAbiType)]
    pub enum stdStorageEvents {
        SlotFoundFilter(SlotFoundFilter),
        WarningUninitedSlotFilter(WarningUninitedSlotFilter),
    }
    impl ethers::contract::EthLogDecode for stdStorageEvents {
        fn decode_log(
            log: &ethers::core::abi::RawLog,
        ) -> ::std::result::Result<Self, ethers::core::abi::Error>
        where
            Self: Sized,
        {
            if let Ok(decoded) = SlotFoundFilter::decode_log(log) {
                return Ok(stdStorageEvents::SlotFoundFilter(decoded));
            }
            if let Ok(decoded) = WarningUninitedSlotFilter::decode_log(log) {
                return Ok(stdStorageEvents::WarningUninitedSlotFilter(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData)
        }
    }
    impl ::std::fmt::Display for stdStorageEvents {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                stdStorageEvents::SlotFoundFilter(element) => element.fmt(f),
                stdStorageEvents::WarningUninitedSlotFilter(element) => element.fmt(f),
            }
        }
    }
    #[doc = "Container type for all input parameters for the `bytesToBytes32` function with signature `bytesToBytes32(bytes,uint256)` and selector `[83, 88, 73, 57]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "bytesToBytes32", abi = "bytesToBytes32(bytes,uint256)")]
    pub struct BytesToBytes32Call {
        pub b: ethers::core::types::Bytes,
        pub offset: ethers::core::types::U256,
    }
    #[doc = "Container type for all return fields from the `bytesToBytes32` function with signature `bytesToBytes32(bytes,uint256)` and selector `[83, 88, 73, 57]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct BytesToBytes32Return(pub [u8; 32]);
}
