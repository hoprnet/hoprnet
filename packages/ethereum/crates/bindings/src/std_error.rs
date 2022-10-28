pub use std_error::*;
#[allow(clippy::too_many_arguments, non_camel_case_types)]
pub mod std_error {
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
    #[doc = "stdError was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    # [rustfmt :: skip] const __ABI : & str = "[{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"arithmeticError\",\"outputs\":[{\"internalType\":\"bytes\",\"name\":\"\",\"type\":\"bytes\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"assertionError\",\"outputs\":[{\"internalType\":\"bytes\",\"name\":\"\",\"type\":\"bytes\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"divisionError\",\"outputs\":[{\"internalType\":\"bytes\",\"name\":\"\",\"type\":\"bytes\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"encodeStorageError\",\"outputs\":[{\"internalType\":\"bytes\",\"name\":\"\",\"type\":\"bytes\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"enumConversionError\",\"outputs\":[{\"internalType\":\"bytes\",\"name\":\"\",\"type\":\"bytes\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"indexOOBError\",\"outputs\":[{\"internalType\":\"bytes\",\"name\":\"\",\"type\":\"bytes\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"lowLevelError\",\"outputs\":[{\"internalType\":\"bytes\",\"name\":\"\",\"type\":\"bytes\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"memOverflowError\",\"outputs\":[{\"internalType\":\"bytes\",\"name\":\"\",\"type\":\"bytes\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"popError\",\"outputs\":[{\"internalType\":\"bytes\",\"name\":\"\",\"type\":\"bytes\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"zeroVarError\",\"outputs\":[{\"internalType\":\"bytes\",\"name\":\"\",\"type\":\"bytes\",\"components\":[]}]}]" ;
    #[doc = r" The parsed JSON-ABI of the contract."]
    pub static STDERROR_ABI: ethers::contract::Lazy<ethers::core::abi::Abi> =
        ethers::contract::Lazy::new(|| {
            ethers::core::utils::__serde_json::from_str(__ABI).expect("invalid abi")
        });
    #[doc = r" Bytecode of the #name contract"]
    pub static STDERROR_BYTECODE: ethers::contract::Lazy<ethers::core::types::Bytes> =
        ethers::contract::Lazy::new(|| {
            "0x61027761003a600b82828239805160001a60731461002d57634e487b7160e01b600052600060045260246000fd5b30600052607381538281f3fe73000000000000000000000000000000000000000030146080604052600436106100a85760003560e01c8063ac3d92c611610070578063ac3d92c6146100eb578063b22dc54d14610101578063b67689da14610109578063d160e4de14610111578063fa784a441461011957600080fd5b806305ee8612146100ad57806310332977146100cb5780631de45560146100d35780638995290f146100db578063986c5f68146100e3575b600080fd5b6100b5610121565b6040516100c291906101ec565b60405180910390f35b6100b561015c565b6100b561016e565b6100b5610180565b6100b5610192565b6100b56040518060200160405280600081525081565b6100b56101a4565b6100b56101b6565b6100b56101c8565b6100b56101da565b604051603260248201526044015b60408051601f198184030181529190526020810180516001600160e01b0316634e487b7160e01b17905281565b6040516001602482015260440161012f565b6040516021602482015260440161012f565b6040516011602482015260440161012f565b6040516041602482015260440161012f565b6040516031602482015260440161012f565b6040516051602482015260440161012f565b6040516022602482015260440161012f565b6040516012602482015260440161012f565b600060208083528351808285015260005b81811015610219578581018301518582016040015282016101fd565b8181111561022b576000604083870101525b50601f01601f191692909201604001939250505056fea2646970667358221220bc5864ab10afc54e49f85b7410683be39101149a38f68a13024dc86a46a6620864736f6c634300080d0033" . parse () . expect ("invalid bytecode")
        });
    pub struct stdError<M>(ethers::contract::Contract<M>);
    impl<M> Clone for stdError<M> {
        fn clone(&self) -> Self {
            stdError(self.0.clone())
        }
    }
    impl<M> std::ops::Deref for stdError<M> {
        type Target = ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> std::fmt::Debug for stdError<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(stdError))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ethers::providers::Middleware> stdError<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            ethers::contract::Contract::new(address.into(), STDERROR_ABI.clone(), client).into()
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
                STDERROR_ABI.clone(),
                STDERROR_BYTECODE.clone().into(),
                client,
            );
            let deployer = factory.deploy(constructor_args)?;
            let deployer = ethers::contract::ContractDeployer::new(deployer);
            Ok(deployer)
        }
        #[doc = "Calls the contract's `arithmeticError` (0x8995290f) function"]
        pub fn arithmetic_error(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Bytes> {
            self.0
                .method_hash([137, 149, 41, 15], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `assertionError` (0x10332977) function"]
        pub fn assertion_error(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Bytes> {
            self.0
                .method_hash([16, 51, 41, 119], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `divisionError` (0xfa784a44) function"]
        pub fn division_error(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Bytes> {
            self.0
                .method_hash([250, 120, 74, 68], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `encodeStorageError` (0xd160e4de) function"]
        pub fn encode_storage_error(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Bytes> {
            self.0
                .method_hash([209, 96, 228, 222], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `enumConversionError` (0x1de45560) function"]
        pub fn enum_conversion_error(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Bytes> {
            self.0
                .method_hash([29, 228, 85, 96], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `indexOOBError` (0x05ee8612) function"]
        pub fn index_oob_error(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Bytes> {
            self.0
                .method_hash([5, 238, 134, 18], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `lowLevelError` (0xac3d92c6) function"]
        pub fn low_level_error(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Bytes> {
            self.0
                .method_hash([172, 61, 146, 198], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `memOverflowError` (0x986c5f68) function"]
        pub fn mem_overflow_error(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Bytes> {
            self.0
                .method_hash([152, 108, 95, 104], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `popError` (0xb22dc54d) function"]
        pub fn pop_error(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Bytes> {
            self.0
                .method_hash([178, 45, 197, 77], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `zeroVarError` (0xb67689da) function"]
        pub fn zero_var_error(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Bytes> {
            self.0
                .method_hash([182, 118, 137, 218], ())
                .expect("method not found (this should never happen)")
        }
    }
    impl<M: ethers::providers::Middleware> From<ethers::contract::Contract<M>> for stdError<M> {
        fn from(contract: ethers::contract::Contract<M>) -> Self {
            Self(contract)
        }
    }
    #[doc = "Container type for all input parameters for the `arithmeticError` function with signature `arithmeticError()` and selector `[137, 149, 41, 15]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "arithmeticError", abi = "arithmeticError()")]
    pub struct ArithmeticErrorCall;
    #[doc = "Container type for all input parameters for the `assertionError` function with signature `assertionError()` and selector `[16, 51, 41, 119]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "assertionError", abi = "assertionError()")]
    pub struct AssertionErrorCall;
    #[doc = "Container type for all input parameters for the `divisionError` function with signature `divisionError()` and selector `[250, 120, 74, 68]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "divisionError", abi = "divisionError()")]
    pub struct DivisionErrorCall;
    #[doc = "Container type for all input parameters for the `encodeStorageError` function with signature `encodeStorageError()` and selector `[209, 96, 228, 222]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "encodeStorageError", abi = "encodeStorageError()")]
    pub struct EncodeStorageErrorCall;
    #[doc = "Container type for all input parameters for the `enumConversionError` function with signature `enumConversionError()` and selector `[29, 228, 85, 96]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "enumConversionError", abi = "enumConversionError()")]
    pub struct EnumConversionErrorCall;
    #[doc = "Container type for all input parameters for the `indexOOBError` function with signature `indexOOBError()` and selector `[5, 238, 134, 18]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "indexOOBError", abi = "indexOOBError()")]
    pub struct IndexOOBErrorCall;
    #[doc = "Container type for all input parameters for the `lowLevelError` function with signature `lowLevelError()` and selector `[172, 61, 146, 198]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "lowLevelError", abi = "lowLevelError()")]
    pub struct LowLevelErrorCall;
    #[doc = "Container type for all input parameters for the `memOverflowError` function with signature `memOverflowError()` and selector `[152, 108, 95, 104]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "memOverflowError", abi = "memOverflowError()")]
    pub struct MemOverflowErrorCall;
    #[doc = "Container type for all input parameters for the `popError` function with signature `popError()` and selector `[178, 45, 197, 77]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "popError", abi = "popError()")]
    pub struct PopErrorCall;
    #[doc = "Container type for all input parameters for the `zeroVarError` function with signature `zeroVarError()` and selector `[182, 118, 137, 218]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "zeroVarError", abi = "zeroVarError()")]
    pub struct ZeroVarErrorCall;
    #[derive(Debug, Clone, PartialEq, Eq, ethers :: contract :: EthAbiType)]
    pub enum stdErrorCalls {
        ArithmeticError(ArithmeticErrorCall),
        AssertionError(AssertionErrorCall),
        DivisionError(DivisionErrorCall),
        EncodeStorageError(EncodeStorageErrorCall),
        EnumConversionError(EnumConversionErrorCall),
        IndexOOBError(IndexOOBErrorCall),
        LowLevelError(LowLevelErrorCall),
        MemOverflowError(MemOverflowErrorCall),
        PopError(PopErrorCall),
        ZeroVarError(ZeroVarErrorCall),
    }
    impl ethers::core::abi::AbiDecode for stdErrorCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::std::result::Result<Self, ethers::core::abi::AbiError> {
            if let Ok(decoded) =
                <ArithmeticErrorCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(stdErrorCalls::ArithmeticError(decoded));
            }
            if let Ok(decoded) =
                <AssertionErrorCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(stdErrorCalls::AssertionError(decoded));
            }
            if let Ok(decoded) =
                <DivisionErrorCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(stdErrorCalls::DivisionError(decoded));
            }
            if let Ok(decoded) =
                <EncodeStorageErrorCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(stdErrorCalls::EncodeStorageError(decoded));
            }
            if let Ok(decoded) =
                <EnumConversionErrorCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(stdErrorCalls::EnumConversionError(decoded));
            }
            if let Ok(decoded) =
                <IndexOOBErrorCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(stdErrorCalls::IndexOOBError(decoded));
            }
            if let Ok(decoded) =
                <LowLevelErrorCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(stdErrorCalls::LowLevelError(decoded));
            }
            if let Ok(decoded) =
                <MemOverflowErrorCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(stdErrorCalls::MemOverflowError(decoded));
            }
            if let Ok(decoded) =
                <PopErrorCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(stdErrorCalls::PopError(decoded));
            }
            if let Ok(decoded) =
                <ZeroVarErrorCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(stdErrorCalls::ZeroVarError(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ethers::core::abi::AbiEncode for stdErrorCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                stdErrorCalls::ArithmeticError(element) => element.encode(),
                stdErrorCalls::AssertionError(element) => element.encode(),
                stdErrorCalls::DivisionError(element) => element.encode(),
                stdErrorCalls::EncodeStorageError(element) => element.encode(),
                stdErrorCalls::EnumConversionError(element) => element.encode(),
                stdErrorCalls::IndexOOBError(element) => element.encode(),
                stdErrorCalls::LowLevelError(element) => element.encode(),
                stdErrorCalls::MemOverflowError(element) => element.encode(),
                stdErrorCalls::PopError(element) => element.encode(),
                stdErrorCalls::ZeroVarError(element) => element.encode(),
            }
        }
    }
    impl ::std::fmt::Display for stdErrorCalls {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                stdErrorCalls::ArithmeticError(element) => element.fmt(f),
                stdErrorCalls::AssertionError(element) => element.fmt(f),
                stdErrorCalls::DivisionError(element) => element.fmt(f),
                stdErrorCalls::EncodeStorageError(element) => element.fmt(f),
                stdErrorCalls::EnumConversionError(element) => element.fmt(f),
                stdErrorCalls::IndexOOBError(element) => element.fmt(f),
                stdErrorCalls::LowLevelError(element) => element.fmt(f),
                stdErrorCalls::MemOverflowError(element) => element.fmt(f),
                stdErrorCalls::PopError(element) => element.fmt(f),
                stdErrorCalls::ZeroVarError(element) => element.fmt(f),
            }
        }
    }
    impl ::std::convert::From<ArithmeticErrorCall> for stdErrorCalls {
        fn from(var: ArithmeticErrorCall) -> Self {
            stdErrorCalls::ArithmeticError(var)
        }
    }
    impl ::std::convert::From<AssertionErrorCall> for stdErrorCalls {
        fn from(var: AssertionErrorCall) -> Self {
            stdErrorCalls::AssertionError(var)
        }
    }
    impl ::std::convert::From<DivisionErrorCall> for stdErrorCalls {
        fn from(var: DivisionErrorCall) -> Self {
            stdErrorCalls::DivisionError(var)
        }
    }
    impl ::std::convert::From<EncodeStorageErrorCall> for stdErrorCalls {
        fn from(var: EncodeStorageErrorCall) -> Self {
            stdErrorCalls::EncodeStorageError(var)
        }
    }
    impl ::std::convert::From<EnumConversionErrorCall> for stdErrorCalls {
        fn from(var: EnumConversionErrorCall) -> Self {
            stdErrorCalls::EnumConversionError(var)
        }
    }
    impl ::std::convert::From<IndexOOBErrorCall> for stdErrorCalls {
        fn from(var: IndexOOBErrorCall) -> Self {
            stdErrorCalls::IndexOOBError(var)
        }
    }
    impl ::std::convert::From<LowLevelErrorCall> for stdErrorCalls {
        fn from(var: LowLevelErrorCall) -> Self {
            stdErrorCalls::LowLevelError(var)
        }
    }
    impl ::std::convert::From<MemOverflowErrorCall> for stdErrorCalls {
        fn from(var: MemOverflowErrorCall) -> Self {
            stdErrorCalls::MemOverflowError(var)
        }
    }
    impl ::std::convert::From<PopErrorCall> for stdErrorCalls {
        fn from(var: PopErrorCall) -> Self {
            stdErrorCalls::PopError(var)
        }
    }
    impl ::std::convert::From<ZeroVarErrorCall> for stdErrorCalls {
        fn from(var: ZeroVarErrorCall) -> Self {
            stdErrorCalls::ZeroVarError(var)
        }
    }
    #[doc = "Container type for all return fields from the `arithmeticError` function with signature `arithmeticError()` and selector `[137, 149, 41, 15]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct ArithmeticErrorReturn(pub ethers::core::types::Bytes);
    #[doc = "Container type for all return fields from the `assertionError` function with signature `assertionError()` and selector `[16, 51, 41, 119]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct AssertionErrorReturn(pub ethers::core::types::Bytes);
    #[doc = "Container type for all return fields from the `divisionError` function with signature `divisionError()` and selector `[250, 120, 74, 68]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct DivisionErrorReturn(pub ethers::core::types::Bytes);
    #[doc = "Container type for all return fields from the `encodeStorageError` function with signature `encodeStorageError()` and selector `[209, 96, 228, 222]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct EncodeStorageErrorReturn(pub ethers::core::types::Bytes);
    #[doc = "Container type for all return fields from the `enumConversionError` function with signature `enumConversionError()` and selector `[29, 228, 85, 96]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct EnumConversionErrorReturn(pub ethers::core::types::Bytes);
    #[doc = "Container type for all return fields from the `indexOOBError` function with signature `indexOOBError()` and selector `[5, 238, 134, 18]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct IndexOOBErrorReturn(pub ethers::core::types::Bytes);
    #[doc = "Container type for all return fields from the `lowLevelError` function with signature `lowLevelError()` and selector `[172, 61, 146, 198]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct LowLevelErrorReturn(pub ethers::core::types::Bytes);
    #[doc = "Container type for all return fields from the `memOverflowError` function with signature `memOverflowError()` and selector `[152, 108, 95, 104]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct MemOverflowErrorReturn(pub ethers::core::types::Bytes);
    #[doc = "Container type for all return fields from the `popError` function with signature `popError()` and selector `[178, 45, 197, 77]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct PopErrorReturn(pub ethers::core::types::Bytes);
    #[doc = "Container type for all return fields from the `zeroVarError` function with signature `zeroVarError()` and selector `[182, 118, 137, 218]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct ZeroVarErrorReturn(pub ethers::core::types::Bytes);
}
