pub use i_burnable_mintable_erc677_token::*;
#[allow(clippy::too_many_arguments, non_camel_case_types)]
pub mod i_burnable_mintable_erc677_token {
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
    #[doc = "IBurnableMintableERC677Token was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    # [rustfmt :: skip] const __ABI : & str = "[{\"inputs\":[{\"name\":\"_spender\",\"type\":\"address\",\"components\":[]},{\"name\":\"_value\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"approve\",\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"totalSupply\",\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"name\":\"_from\",\"type\":\"address\",\"components\":[]},{\"name\":\"_to\",\"type\":\"address\",\"components\":[]},{\"name\":\"_value\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"transferFrom\",\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"name\":\"spender\",\"type\":\"address\",\"components\":[]},{\"name\":\"addedValue\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"increaseAllowance\",\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"name\":\"\",\"type\":\"address\",\"components\":[]},{\"name\":\"\",\"type\":\"uint256\",\"components\":[]},{\"name\":\"\",\"type\":\"bytes\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"transferAndCall\",\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"name\":\"_to\",\"type\":\"address\",\"components\":[]},{\"name\":\"_amount\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"mint\",\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"name\":\"_value\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"burn\",\"outputs\":[]},{\"inputs\":[{\"name\":\"_token\",\"type\":\"address\",\"components\":[]},{\"name\":\"_to\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"claimTokens\",\"outputs\":[]},{\"inputs\":[{\"name\":\"_who\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"balanceOf\",\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"name\":\"spender\",\"type\":\"address\",\"components\":[]},{\"name\":\"subtractedValue\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"decreaseAllowance\",\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"name\":\"_to\",\"type\":\"address\",\"components\":[]},{\"name\":\"_value\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"transfer\",\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"name\":\"_owner\",\"type\":\"address\",\"components\":[]},{\"name\":\"_spender\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"allowance\",\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"name\":\"from\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"name\":\"to\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"name\":\"value\",\"type\":\"uint256\",\"components\":[],\"indexed\":false},{\"name\":\"data\",\"type\":\"bytes\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"Transfer\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"name\":\"owner\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"name\":\"spender\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"name\":\"value\",\"type\":\"uint256\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"Approval\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"name\":\"from\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"name\":\"to\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"name\":\"value\",\"type\":\"uint256\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"Transfer\",\"outputs\":[],\"anonymous\":false}]" ;
    #[doc = r" The parsed JSON-ABI of the contract."]
    pub static IBURNABLEMINTABLEERC677TOKEN_ABI: ethers::contract::Lazy<ethers::core::abi::Abi> =
        ethers::contract::Lazy::new(|| {
            ethers::core::utils::__serde_json::from_str(__ABI).expect("invalid abi")
        });
    pub struct IBurnableMintableERC677Token<M>(ethers::contract::Contract<M>);
    impl<M> Clone for IBurnableMintableERC677Token<M> {
        fn clone(&self) -> Self {
            IBurnableMintableERC677Token(self.0.clone())
        }
    }
    impl<M> std::ops::Deref for IBurnableMintableERC677Token<M> {
        type Target = ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> std::fmt::Debug for IBurnableMintableERC677Token<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(IBurnableMintableERC677Token))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ethers::providers::Middleware> IBurnableMintableERC677Token<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            ethers::contract::Contract::new(
                address.into(),
                IBURNABLEMINTABLEERC677TOKEN_ABI.clone(),
                client,
            )
            .into()
        }
        #[doc = "Calls the contract's `allowance` (0xdd62ed3e) function"]
        pub fn allowance(
            &self,
            owner: ethers::core::types::Address,
            spender: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([221, 98, 237, 62], (owner, spender))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `approve` (0x095ea7b3) function"]
        pub fn approve(
            &self,
            spender: ethers::core::types::Address,
            value: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([9, 94, 167, 179], (spender, value))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `balanceOf` (0x70a08231) function"]
        pub fn balance_of(
            &self,
            who: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([112, 160, 130, 49], who)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `burn` (0x42966c68) function"]
        pub fn burn(
            &self,
            value: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([66, 150, 108, 104], value)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `claimTokens` (0x69ffa08a) function"]
        pub fn claim_tokens(
            &self,
            token: ethers::core::types::Address,
            to: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([105, 255, 160, 138], (token, to))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `decreaseAllowance` (0xa457c2d7) function"]
        pub fn decrease_allowance(
            &self,
            spender: ethers::core::types::Address,
            subtracted_value: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([164, 87, 194, 215], (spender, subtracted_value))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `increaseAllowance` (0x39509351) function"]
        pub fn increase_allowance(
            &self,
            spender: ethers::core::types::Address,
            added_value: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([57, 80, 147, 81], (spender, added_value))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `mint` (0x40c10f19) function"]
        pub fn mint(
            &self,
            to: ethers::core::types::Address,
            amount: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([64, 193, 15, 25], (to, amount))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `totalSupply` (0x18160ddd) function"]
        pub fn total_supply(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([24, 22, 13, 221], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `transfer` (0xa9059cbb) function"]
        pub fn transfer(
            &self,
            to: ethers::core::types::Address,
            value: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([169, 5, 156, 187], (to, value))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `transferAndCall` (0x4000aea0) function"]
        pub fn transfer_and_call(
            &self,
            p0: ethers::core::types::Address,
            p1: ethers::core::types::U256,
            p2: ethers::core::types::Bytes,
        ) -> ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([64, 0, 174, 160], (p0, p1, p2))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `transferFrom` (0x23b872dd) function"]
        pub fn transfer_from(
            &self,
            from: ethers::core::types::Address,
            to: ethers::core::types::Address,
            value: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([35, 184, 114, 221], (from, to, value))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Gets the contract's `Approval` event"]
        pub fn approval_filter(&self) -> ethers::contract::builders::Event<M, ApprovalFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `Transfer` event"]
        pub fn transfer_1_filter(&self) -> ethers::contract::builders::Event<M, Transfer1Filter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `Transfer` event"]
        pub fn transfer_2_filter(&self) -> ethers::contract::builders::Event<M, Transfer2Filter> {
            self.0.event()
        }
        #[doc = r" Returns an [`Event`](#ethers_contract::builders::Event) builder for all events of this contract"]
        pub fn events(
            &self,
        ) -> ethers::contract::builders::Event<M, IBurnableMintableERC677TokenEvents> {
            self.0.event_with_filter(Default::default())
        }
    }
    impl<M: ethers::providers::Middleware> From<ethers::contract::Contract<M>>
        for IBurnableMintableERC677Token<M>
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
    #[ethevent(name = "Approval", abi = "Approval(address,address,uint256)")]
    pub struct ApprovalFilter {
        #[ethevent(indexed)]
        pub owner: ethers::core::types::Address,
        #[ethevent(indexed)]
        pub spender: ethers::core::types::Address,
        pub value: ethers::core::types::U256,
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
    #[ethevent(name = "Transfer", abi = "Transfer(address,address,uint256,bytes)")]
    pub struct Transfer1Filter {
        #[ethevent(indexed)]
        pub from: ethers::core::types::Address,
        #[ethevent(indexed)]
        pub to: ethers::core::types::Address,
        pub value: ethers::core::types::U256,
        pub data: ethers::core::types::Bytes,
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
    #[ethevent(name = "Transfer", abi = "Transfer(address,address,uint256)")]
    pub struct Transfer2Filter {
        #[ethevent(indexed)]
        pub from: ethers::core::types::Address,
        #[ethevent(indexed)]
        pub to: ethers::core::types::Address,
        pub value: ethers::core::types::U256,
    }
    #[derive(Debug, Clone, PartialEq, Eq, ethers :: contract :: EthAbiType)]
    pub enum IBurnableMintableERC677TokenEvents {
        ApprovalFilter(ApprovalFilter),
        Transfer1Filter(Transfer1Filter),
        Transfer2Filter(Transfer2Filter),
    }
    impl ethers::contract::EthLogDecode for IBurnableMintableERC677TokenEvents {
        fn decode_log(
            log: &ethers::core::abi::RawLog,
        ) -> ::std::result::Result<Self, ethers::core::abi::Error>
        where
            Self: Sized,
        {
            if let Ok(decoded) = ApprovalFilter::decode_log(log) {
                return Ok(IBurnableMintableERC677TokenEvents::ApprovalFilter(decoded));
            }
            if let Ok(decoded) = Transfer1Filter::decode_log(log) {
                return Ok(IBurnableMintableERC677TokenEvents::Transfer1Filter(decoded));
            }
            if let Ok(decoded) = Transfer2Filter::decode_log(log) {
                return Ok(IBurnableMintableERC677TokenEvents::Transfer2Filter(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData)
        }
    }
    impl ::std::fmt::Display for IBurnableMintableERC677TokenEvents {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                IBurnableMintableERC677TokenEvents::ApprovalFilter(element) => element.fmt(f),
                IBurnableMintableERC677TokenEvents::Transfer1Filter(element) => element.fmt(f),
                IBurnableMintableERC677TokenEvents::Transfer2Filter(element) => element.fmt(f),
            }
        }
    }
    #[doc = "Container type for all input parameters for the `allowance` function with signature `allowance(address,address)` and selector `[221, 98, 237, 62]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "allowance", abi = "allowance(address,address)")]
    pub struct AllowanceCall {
        pub owner: ethers::core::types::Address,
        pub spender: ethers::core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `approve` function with signature `approve(address,uint256)` and selector `[9, 94, 167, 179]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "approve", abi = "approve(address,uint256)")]
    pub struct ApproveCall {
        pub spender: ethers::core::types::Address,
        pub value: ethers::core::types::U256,
    }
    #[doc = "Container type for all input parameters for the `balanceOf` function with signature `balanceOf(address)` and selector `[112, 160, 130, 49]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "balanceOf", abi = "balanceOf(address)")]
    pub struct BalanceOfCall {
        pub who: ethers::core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `burn` function with signature `burn(uint256)` and selector `[66, 150, 108, 104]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "burn", abi = "burn(uint256)")]
    pub struct BurnCall {
        pub value: ethers::core::types::U256,
    }
    #[doc = "Container type for all input parameters for the `claimTokens` function with signature `claimTokens(address,address)` and selector `[105, 255, 160, 138]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "claimTokens", abi = "claimTokens(address,address)")]
    pub struct ClaimTokensCall {
        pub token: ethers::core::types::Address,
        pub to: ethers::core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `decreaseAllowance` function with signature `decreaseAllowance(address,uint256)` and selector `[164, 87, 194, 215]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "decreaseAllowance", abi = "decreaseAllowance(address,uint256)")]
    pub struct DecreaseAllowanceCall {
        pub spender: ethers::core::types::Address,
        pub subtracted_value: ethers::core::types::U256,
    }
    #[doc = "Container type for all input parameters for the `increaseAllowance` function with signature `increaseAllowance(address,uint256)` and selector `[57, 80, 147, 81]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "increaseAllowance", abi = "increaseAllowance(address,uint256)")]
    pub struct IncreaseAllowanceCall {
        pub spender: ethers::core::types::Address,
        pub added_value: ethers::core::types::U256,
    }
    #[doc = "Container type for all input parameters for the `mint` function with signature `mint(address,uint256)` and selector `[64, 193, 15, 25]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "mint", abi = "mint(address,uint256)")]
    pub struct MintCall {
        pub to: ethers::core::types::Address,
        pub amount: ethers::core::types::U256,
    }
    #[doc = "Container type for all input parameters for the `totalSupply` function with signature `totalSupply()` and selector `[24, 22, 13, 221]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "totalSupply", abi = "totalSupply()")]
    pub struct TotalSupplyCall;
    #[doc = "Container type for all input parameters for the `transfer` function with signature `transfer(address,uint256)` and selector `[169, 5, 156, 187]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "transfer", abi = "transfer(address,uint256)")]
    pub struct TransferCall {
        pub to: ethers::core::types::Address,
        pub value: ethers::core::types::U256,
    }
    #[doc = "Container type for all input parameters for the `transferAndCall` function with signature `transferAndCall(address,uint256,bytes)` and selector `[64, 0, 174, 160]`"]
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
        name = "transferAndCall",
        abi = "transferAndCall(address,uint256,bytes)"
    )]
    pub struct TransferAndCallCall(
        pub ethers::core::types::Address,
        pub ethers::core::types::U256,
        pub ethers::core::types::Bytes,
    );
    #[doc = "Container type for all input parameters for the `transferFrom` function with signature `transferFrom(address,address,uint256)` and selector `[35, 184, 114, 221]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "transferFrom", abi = "transferFrom(address,address,uint256)")]
    pub struct TransferFromCall {
        pub from: ethers::core::types::Address,
        pub to: ethers::core::types::Address,
        pub value: ethers::core::types::U256,
    }
    #[derive(Debug, Clone, PartialEq, Eq, ethers :: contract :: EthAbiType)]
    pub enum IBurnableMintableERC677TokenCalls {
        Allowance(AllowanceCall),
        Approve(ApproveCall),
        BalanceOf(BalanceOfCall),
        Burn(BurnCall),
        ClaimTokens(ClaimTokensCall),
        DecreaseAllowance(DecreaseAllowanceCall),
        IncreaseAllowance(IncreaseAllowanceCall),
        Mint(MintCall),
        TotalSupply(TotalSupplyCall),
        Transfer(TransferCall),
        TransferAndCall(TransferAndCallCall),
        TransferFrom(TransferFromCall),
    }
    impl ethers::core::abi::AbiDecode for IBurnableMintableERC677TokenCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::std::result::Result<Self, ethers::core::abi::AbiError> {
            if let Ok(decoded) =
                <AllowanceCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IBurnableMintableERC677TokenCalls::Allowance(decoded));
            }
            if let Ok(decoded) =
                <ApproveCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IBurnableMintableERC677TokenCalls::Approve(decoded));
            }
            if let Ok(decoded) =
                <BalanceOfCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IBurnableMintableERC677TokenCalls::BalanceOf(decoded));
            }
            if let Ok(decoded) = <BurnCall as ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(IBurnableMintableERC677TokenCalls::Burn(decoded));
            }
            if let Ok(decoded) =
                <ClaimTokensCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IBurnableMintableERC677TokenCalls::ClaimTokens(decoded));
            }
            if let Ok(decoded) =
                <DecreaseAllowanceCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IBurnableMintableERC677TokenCalls::DecreaseAllowance(
                    decoded,
                ));
            }
            if let Ok(decoded) =
                <IncreaseAllowanceCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IBurnableMintableERC677TokenCalls::IncreaseAllowance(
                    decoded,
                ));
            }
            if let Ok(decoded) = <MintCall as ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(IBurnableMintableERC677TokenCalls::Mint(decoded));
            }
            if let Ok(decoded) =
                <TotalSupplyCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IBurnableMintableERC677TokenCalls::TotalSupply(decoded));
            }
            if let Ok(decoded) =
                <TransferCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IBurnableMintableERC677TokenCalls::Transfer(decoded));
            }
            if let Ok(decoded) =
                <TransferAndCallCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IBurnableMintableERC677TokenCalls::TransferAndCall(decoded));
            }
            if let Ok(decoded) =
                <TransferFromCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IBurnableMintableERC677TokenCalls::TransferFrom(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ethers::core::abi::AbiEncode for IBurnableMintableERC677TokenCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                IBurnableMintableERC677TokenCalls::Allowance(element) => element.encode(),
                IBurnableMintableERC677TokenCalls::Approve(element) => element.encode(),
                IBurnableMintableERC677TokenCalls::BalanceOf(element) => element.encode(),
                IBurnableMintableERC677TokenCalls::Burn(element) => element.encode(),
                IBurnableMintableERC677TokenCalls::ClaimTokens(element) => element.encode(),
                IBurnableMintableERC677TokenCalls::DecreaseAllowance(element) => element.encode(),
                IBurnableMintableERC677TokenCalls::IncreaseAllowance(element) => element.encode(),
                IBurnableMintableERC677TokenCalls::Mint(element) => element.encode(),
                IBurnableMintableERC677TokenCalls::TotalSupply(element) => element.encode(),
                IBurnableMintableERC677TokenCalls::Transfer(element) => element.encode(),
                IBurnableMintableERC677TokenCalls::TransferAndCall(element) => element.encode(),
                IBurnableMintableERC677TokenCalls::TransferFrom(element) => element.encode(),
            }
        }
    }
    impl ::std::fmt::Display for IBurnableMintableERC677TokenCalls {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                IBurnableMintableERC677TokenCalls::Allowance(element) => element.fmt(f),
                IBurnableMintableERC677TokenCalls::Approve(element) => element.fmt(f),
                IBurnableMintableERC677TokenCalls::BalanceOf(element) => element.fmt(f),
                IBurnableMintableERC677TokenCalls::Burn(element) => element.fmt(f),
                IBurnableMintableERC677TokenCalls::ClaimTokens(element) => element.fmt(f),
                IBurnableMintableERC677TokenCalls::DecreaseAllowance(element) => element.fmt(f),
                IBurnableMintableERC677TokenCalls::IncreaseAllowance(element) => element.fmt(f),
                IBurnableMintableERC677TokenCalls::Mint(element) => element.fmt(f),
                IBurnableMintableERC677TokenCalls::TotalSupply(element) => element.fmt(f),
                IBurnableMintableERC677TokenCalls::Transfer(element) => element.fmt(f),
                IBurnableMintableERC677TokenCalls::TransferAndCall(element) => element.fmt(f),
                IBurnableMintableERC677TokenCalls::TransferFrom(element) => element.fmt(f),
            }
        }
    }
    impl ::std::convert::From<AllowanceCall> for IBurnableMintableERC677TokenCalls {
        fn from(var: AllowanceCall) -> Self {
            IBurnableMintableERC677TokenCalls::Allowance(var)
        }
    }
    impl ::std::convert::From<ApproveCall> for IBurnableMintableERC677TokenCalls {
        fn from(var: ApproveCall) -> Self {
            IBurnableMintableERC677TokenCalls::Approve(var)
        }
    }
    impl ::std::convert::From<BalanceOfCall> for IBurnableMintableERC677TokenCalls {
        fn from(var: BalanceOfCall) -> Self {
            IBurnableMintableERC677TokenCalls::BalanceOf(var)
        }
    }
    impl ::std::convert::From<BurnCall> for IBurnableMintableERC677TokenCalls {
        fn from(var: BurnCall) -> Self {
            IBurnableMintableERC677TokenCalls::Burn(var)
        }
    }
    impl ::std::convert::From<ClaimTokensCall> for IBurnableMintableERC677TokenCalls {
        fn from(var: ClaimTokensCall) -> Self {
            IBurnableMintableERC677TokenCalls::ClaimTokens(var)
        }
    }
    impl ::std::convert::From<DecreaseAllowanceCall> for IBurnableMintableERC677TokenCalls {
        fn from(var: DecreaseAllowanceCall) -> Self {
            IBurnableMintableERC677TokenCalls::DecreaseAllowance(var)
        }
    }
    impl ::std::convert::From<IncreaseAllowanceCall> for IBurnableMintableERC677TokenCalls {
        fn from(var: IncreaseAllowanceCall) -> Self {
            IBurnableMintableERC677TokenCalls::IncreaseAllowance(var)
        }
    }
    impl ::std::convert::From<MintCall> for IBurnableMintableERC677TokenCalls {
        fn from(var: MintCall) -> Self {
            IBurnableMintableERC677TokenCalls::Mint(var)
        }
    }
    impl ::std::convert::From<TotalSupplyCall> for IBurnableMintableERC677TokenCalls {
        fn from(var: TotalSupplyCall) -> Self {
            IBurnableMintableERC677TokenCalls::TotalSupply(var)
        }
    }
    impl ::std::convert::From<TransferCall> for IBurnableMintableERC677TokenCalls {
        fn from(var: TransferCall) -> Self {
            IBurnableMintableERC677TokenCalls::Transfer(var)
        }
    }
    impl ::std::convert::From<TransferAndCallCall> for IBurnableMintableERC677TokenCalls {
        fn from(var: TransferAndCallCall) -> Self {
            IBurnableMintableERC677TokenCalls::TransferAndCall(var)
        }
    }
    impl ::std::convert::From<TransferFromCall> for IBurnableMintableERC677TokenCalls {
        fn from(var: TransferFromCall) -> Self {
            IBurnableMintableERC677TokenCalls::TransferFrom(var)
        }
    }
    #[doc = "Container type for all return fields from the `allowance` function with signature `allowance(address,address)` and selector `[221, 98, 237, 62]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct AllowanceReturn(pub ethers::core::types::U256);
    #[doc = "Container type for all return fields from the `approve` function with signature `approve(address,uint256)` and selector `[9, 94, 167, 179]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct ApproveReturn(pub bool);
    #[doc = "Container type for all return fields from the `balanceOf` function with signature `balanceOf(address)` and selector `[112, 160, 130, 49]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct BalanceOfReturn(pub ethers::core::types::U256);
    #[doc = "Container type for all return fields from the `decreaseAllowance` function with signature `decreaseAllowance(address,uint256)` and selector `[164, 87, 194, 215]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct DecreaseAllowanceReturn(pub bool);
    #[doc = "Container type for all return fields from the `increaseAllowance` function with signature `increaseAllowance(address,uint256)` and selector `[57, 80, 147, 81]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct IncreaseAllowanceReturn(pub bool);
    #[doc = "Container type for all return fields from the `mint` function with signature `mint(address,uint256)` and selector `[64, 193, 15, 25]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct MintReturn(pub bool);
    #[doc = "Container type for all return fields from the `totalSupply` function with signature `totalSupply()` and selector `[24, 22, 13, 221]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct TotalSupplyReturn(pub ethers::core::types::U256);
    #[doc = "Container type for all return fields from the `transfer` function with signature `transfer(address,uint256)` and selector `[169, 5, 156, 187]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct TransferReturn(pub bool);
    #[doc = "Container type for all return fields from the `transferAndCall` function with signature `transferAndCall(address,uint256,bytes)` and selector `[64, 0, 174, 160]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct TransferAndCallReturn(pub bool);
    #[doc = "Container type for all return fields from the `transferFrom` function with signature `transferFrom(address,address,uint256)` and selector `[35, 184, 114, 221]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct TransferFromReturn(pub bool);
}
