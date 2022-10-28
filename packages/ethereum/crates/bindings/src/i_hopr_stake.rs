pub use i_hopr_stake::*;
#[allow(clippy::too_many_arguments, non_camel_case_types)]
pub mod i_hopr_stake {
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
    #[doc = "IHoprStake was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    # [rustfmt :: skip] const __ABI : & str = "[{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"nftTypeIndex\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"nftRank\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"hodler\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"isNftTypeAndRankRedeemed2\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"_account\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"stakedHoprTokens\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]}]" ;
    #[doc = r" The parsed JSON-ABI of the contract."]
    pub static IHOPRSTAKE_ABI: ethers::contract::Lazy<ethers::core::abi::Abi> =
        ethers::contract::Lazy::new(|| {
            ethers::core::utils::__serde_json::from_str(__ABI).expect("invalid abi")
        });
    #[doc = r" Bytecode of the #name contract"]
    pub static IHOPRSTAKE_BYTECODE: ethers::contract::Lazy<ethers::core::types::Bytes> =
        ethers::contract::Lazy::new(|| {
            "0x608060405234801561001057600080fd5b506101de806100206000396000f3fe608060405234801561001057600080fd5b50600436106100365760003560e01c806396a9cd7d1461003b578063f978fff114610067575b600080fd5b6100526100493660046100bb565b60009392505050565b60405190151581526020015b60405180910390f35b61007b610075366004610186565b50600090565b60405190815260200161005e565b634e487b7160e01b600052604160045260246000fd5b80356001600160a01b03811681146100b657600080fd5b919050565b6000806000606084860312156100d057600080fd5b83359250602084013567ffffffffffffffff808211156100ef57600080fd5b818601915086601f83011261010357600080fd5b81358181111561011557610115610089565b604051601f8201601f19908116603f0116810190838211818310171561013d5761013d610089565b8160405282815289602084870101111561015657600080fd5b82602086016020830137600060208483010152809650505050505061017d6040850161009f565b90509250925092565b60006020828403121561019857600080fd5b6101a18261009f565b939250505056fea26469706673582212207492397810635ab6d88ca7543e888bcd648f4a0c7e8abe0a10f27d9d2c2285e964736f6c634300080d0033" . parse () . expect ("invalid bytecode")
        });
    pub struct IHoprStake<M>(ethers::contract::Contract<M>);
    impl<M> Clone for IHoprStake<M> {
        fn clone(&self) -> Self {
            IHoprStake(self.0.clone())
        }
    }
    impl<M> std::ops::Deref for IHoprStake<M> {
        type Target = ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> std::fmt::Debug for IHoprStake<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(IHoprStake))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ethers::providers::Middleware> IHoprStake<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            ethers::contract::Contract::new(address.into(), IHOPRSTAKE_ABI.clone(), client).into()
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
                IHOPRSTAKE_ABI.clone(),
                IHOPRSTAKE_BYTECODE.clone().into(),
                client,
            );
            let deployer = factory.deploy(constructor_args)?;
            let deployer = ethers::contract::ContractDeployer::new(deployer);
            Ok(deployer)
        }
        #[doc = "Calls the contract's `isNftTypeAndRankRedeemed2` (0x96a9cd7d) function"]
        pub fn is_nft_type_and_rank_redeemed_2(
            &self,
            nft_type_index: ethers::core::types::U256,
            nft_rank: String,
            hodler: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([150, 169, 205, 125], (nft_type_index, nft_rank, hodler))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `stakedHoprTokens` (0xf978fff1) function"]
        pub fn staked_hopr_tokens(
            &self,
            account: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([249, 120, 255, 241], account)
                .expect("method not found (this should never happen)")
        }
    }
    impl<M: ethers::providers::Middleware> From<ethers::contract::Contract<M>> for IHoprStake<M> {
        fn from(contract: ethers::contract::Contract<M>) -> Self {
            Self(contract)
        }
    }
    #[doc = "Container type for all input parameters for the `isNftTypeAndRankRedeemed2` function with signature `isNftTypeAndRankRedeemed2(uint256,string,address)` and selector `[150, 169, 205, 125]`"]
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
        name = "isNftTypeAndRankRedeemed2",
        abi = "isNftTypeAndRankRedeemed2(uint256,string,address)"
    )]
    pub struct IsNftTypeAndRankRedeemed2Call {
        pub nft_type_index: ethers::core::types::U256,
        pub nft_rank: String,
        pub hodler: ethers::core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `stakedHoprTokens` function with signature `stakedHoprTokens(address)` and selector `[249, 120, 255, 241]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "stakedHoprTokens", abi = "stakedHoprTokens(address)")]
    pub struct StakedHoprTokensCall {
        pub account: ethers::core::types::Address,
    }
    #[derive(Debug, Clone, PartialEq, Eq, ethers :: contract :: EthAbiType)]
    pub enum IHoprStakeCalls {
        IsNftTypeAndRankRedeemed2(IsNftTypeAndRankRedeemed2Call),
        StakedHoprTokens(StakedHoprTokensCall),
    }
    impl ethers::core::abi::AbiDecode for IHoprStakeCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::std::result::Result<Self, ethers::core::abi::AbiError> {
            if let Ok(decoded) =
                <IsNftTypeAndRankRedeemed2Call as ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                )
            {
                return Ok(IHoprStakeCalls::IsNftTypeAndRankRedeemed2(decoded));
            }
            if let Ok(decoded) =
                <StakedHoprTokensCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IHoprStakeCalls::StakedHoprTokens(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ethers::core::abi::AbiEncode for IHoprStakeCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                IHoprStakeCalls::IsNftTypeAndRankRedeemed2(element) => element.encode(),
                IHoprStakeCalls::StakedHoprTokens(element) => element.encode(),
            }
        }
    }
    impl ::std::fmt::Display for IHoprStakeCalls {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                IHoprStakeCalls::IsNftTypeAndRankRedeemed2(element) => element.fmt(f),
                IHoprStakeCalls::StakedHoprTokens(element) => element.fmt(f),
            }
        }
    }
    impl ::std::convert::From<IsNftTypeAndRankRedeemed2Call> for IHoprStakeCalls {
        fn from(var: IsNftTypeAndRankRedeemed2Call) -> Self {
            IHoprStakeCalls::IsNftTypeAndRankRedeemed2(var)
        }
    }
    impl ::std::convert::From<StakedHoprTokensCall> for IHoprStakeCalls {
        fn from(var: StakedHoprTokensCall) -> Self {
            IHoprStakeCalls::StakedHoprTokens(var)
        }
    }
    #[doc = "Container type for all return fields from the `isNftTypeAndRankRedeemed2` function with signature `isNftTypeAndRankRedeemed2(uint256,string,address)` and selector `[150, 169, 205, 125]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct IsNftTypeAndRankRedeemed2Return(pub bool);
    #[doc = "Container type for all return fields from the `stakedHoprTokens` function with signature `stakedHoprTokens(address)` and selector `[249, 120, 255, 241]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct StakedHoprTokensReturn(pub ethers::core::types::U256);
}
