pub use erc777_snapshot::*;
#[allow(clippy::too_many_arguments, non_camel_case_types)]
pub mod erc777_snapshot {
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
    #[doc = "ERC777Snapshot was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    # [rustfmt :: skip] const __ABI : & str = "[{\"inputs\":[{\"internalType\":\"address\",\"name\":\"owner\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"spender\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"uint256\",\"name\":\"value\",\"type\":\"uint256\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"Approval\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"tokenHolder\",\"type\":\"address\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"AuthorizedOperator\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"from\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[],\"indexed\":false},{\"internalType\":\"bytes\",\"name\":\"data\",\"type\":\"bytes\",\"components\":[],\"indexed\":false},{\"internalType\":\"bytes\",\"name\":\"operatorData\",\"type\":\"bytes\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"Burned\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[],\"indexed\":false},{\"internalType\":\"bytes\",\"name\":\"data\",\"type\":\"bytes\",\"components\":[],\"indexed\":false},{\"internalType\":\"bytes\",\"name\":\"operatorData\",\"type\":\"bytes\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"Minted\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"tokenHolder\",\"type\":\"address\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"RevokedOperator\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"from\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[],\"indexed\":false},{\"internalType\":\"bytes\",\"name\":\"data\",\"type\":\"bytes\",\"components\":[],\"indexed\":false},{\"internalType\":\"bytes\",\"name\":\"operatorData\",\"type\":\"bytes\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"Sent\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"from\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"uint256\",\"name\":\"value\",\"type\":\"uint256\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"Transfer\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"accountSnapshots\",\"outputs\":[{\"internalType\":\"uint128\",\"name\":\"fromBlock\",\"type\":\"uint128\",\"components\":[]},{\"internalType\":\"uint128\",\"name\":\"value\",\"type\":\"uint128\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"holder\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"spender\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"allowance\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"spender\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"value\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"approve\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"authorizeOperator\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"tokenHolder\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"balanceOf\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"_owner\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint128\",\"name\":\"_blockNumber\",\"type\":\"uint128\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"balanceOfAt\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"data\",\"type\":\"bytes\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"burn\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"pure\",\"type\":\"function\",\"name\":\"decimals\",\"outputs\":[{\"internalType\":\"uint8\",\"name\":\"\",\"type\":\"uint8\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"defaultOperators\",\"outputs\":[{\"internalType\":\"address[]\",\"name\":\"\",\"type\":\"address[]\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"granularity\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"tokenHolder\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"isOperatorFor\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"name\",\"outputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"data\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"operatorData\",\"type\":\"bytes\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"operatorBurn\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"sender\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"recipient\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"data\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"operatorData\",\"type\":\"bytes\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"operatorSend\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"revokeOperator\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"recipient\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"data\",\"type\":\"bytes\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"send\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"symbol\",\"outputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"totalSupply\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"uint128\",\"name\":\"_blockNumber\",\"type\":\"uint128\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"totalSupplyAt\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"totalSupplySnapshots\",\"outputs\":[{\"internalType\":\"uint128\",\"name\":\"fromBlock\",\"type\":\"uint128\",\"components\":[]},{\"internalType\":\"uint128\",\"name\":\"value\",\"type\":\"uint128\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"recipient\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"transfer\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"holder\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"recipient\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"transferFrom\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]}]" ;
    #[doc = r" The parsed JSON-ABI of the contract."]
    pub static ERC777SNAPSHOT_ABI: ethers::contract::Lazy<ethers::core::abi::Abi> =
        ethers::contract::Lazy::new(|| {
            ethers::core::utils::__serde_json::from_str(__ABI).expect("invalid abi")
        });
    pub struct ERC777Snapshot<M>(ethers::contract::Contract<M>);
    impl<M> Clone for ERC777Snapshot<M> {
        fn clone(&self) -> Self {
            ERC777Snapshot(self.0.clone())
        }
    }
    impl<M> std::ops::Deref for ERC777Snapshot<M> {
        type Target = ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> std::fmt::Debug for ERC777Snapshot<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(ERC777Snapshot))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ethers::providers::Middleware> ERC777Snapshot<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            ethers::contract::Contract::new(address.into(), ERC777SNAPSHOT_ABI.clone(), client)
                .into()
        }
        #[doc = "Calls the contract's `accountSnapshots` (0x2497aee6) function"]
        pub fn account_snapshots(
            &self,
            p0: ethers::core::types::Address,
            p1: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, (u128, u128)> {
            self.0
                .method_hash([36, 151, 174, 230], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `allowance` (0xdd62ed3e) function"]
        pub fn allowance(
            &self,
            holder: ethers::core::types::Address,
            spender: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([221, 98, 237, 62], (holder, spender))
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
        #[doc = "Calls the contract's `authorizeOperator` (0x959b8c3f) function"]
        pub fn authorize_operator(
            &self,
            operator: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([149, 155, 140, 63], operator)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `balanceOf` (0x70a08231) function"]
        pub fn balance_of(
            &self,
            token_holder: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([112, 160, 130, 49], token_holder)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `balanceOfAt` (0xf772a092) function"]
        pub fn balance_of_at(
            &self,
            owner: ethers::core::types::Address,
            block_number: u128,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([247, 114, 160, 146], (owner, block_number))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `burn` (0xfe9d9303) function"]
        pub fn burn(
            &self,
            amount: ethers::core::types::U256,
            data: ethers::core::types::Bytes,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([254, 157, 147, 3], (amount, data))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `decimals` (0x313ce567) function"]
        pub fn decimals(&self) -> ethers::contract::builders::ContractCall<M, u8> {
            self.0
                .method_hash([49, 60, 229, 103], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `defaultOperators` (0x06e48538) function"]
        pub fn default_operators(
            &self,
        ) -> ethers::contract::builders::ContractCall<
            M,
            ::std::vec::Vec<ethers::core::types::Address>,
        > {
            self.0
                .method_hash([6, 228, 133, 56], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `granularity` (0x556f0dc7) function"]
        pub fn granularity(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([85, 111, 13, 199], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `isOperatorFor` (0xd95b6371) function"]
        pub fn is_operator_for(
            &self,
            operator: ethers::core::types::Address,
            token_holder: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([217, 91, 99, 113], (operator, token_holder))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `name` (0x06fdde03) function"]
        pub fn name(&self) -> ethers::contract::builders::ContractCall<M, String> {
            self.0
                .method_hash([6, 253, 222, 3], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `operatorBurn` (0xfc673c4f) function"]
        pub fn operator_burn(
            &self,
            account: ethers::core::types::Address,
            amount: ethers::core::types::U256,
            data: ethers::core::types::Bytes,
            operator_data: ethers::core::types::Bytes,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([252, 103, 60, 79], (account, amount, data, operator_data))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `operatorSend` (0x62ad1b83) function"]
        pub fn operator_send(
            &self,
            sender: ethers::core::types::Address,
            recipient: ethers::core::types::Address,
            amount: ethers::core::types::U256,
            data: ethers::core::types::Bytes,
            operator_data: ethers::core::types::Bytes,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [98, 173, 27, 131],
                    (sender, recipient, amount, data, operator_data),
                )
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `revokeOperator` (0xfad8b32a) function"]
        pub fn revoke_operator(
            &self,
            operator: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([250, 216, 179, 42], operator)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `send` (0x9bd9bbc6) function"]
        pub fn send(
            &self,
            recipient: ethers::core::types::Address,
            amount: ethers::core::types::U256,
            data: ethers::core::types::Bytes,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([155, 217, 187, 198], (recipient, amount, data))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `symbol` (0x95d89b41) function"]
        pub fn symbol(&self) -> ethers::contract::builders::ContractCall<M, String> {
            self.0
                .method_hash([149, 216, 155, 65], ())
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
        #[doc = "Calls the contract's `totalSupplyAt` (0x947975d9) function"]
        pub fn total_supply_at(
            &self,
            block_number: u128,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([148, 121, 117, 217], block_number)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `totalSupplySnapshots` (0xb7d78b1a) function"]
        pub fn total_supply_snapshots(
            &self,
            p0: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, (u128, u128)> {
            self.0
                .method_hash([183, 215, 139, 26], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `transfer` (0xa9059cbb) function"]
        pub fn transfer(
            &self,
            recipient: ethers::core::types::Address,
            amount: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([169, 5, 156, 187], (recipient, amount))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `transferFrom` (0x23b872dd) function"]
        pub fn transfer_from(
            &self,
            holder: ethers::core::types::Address,
            recipient: ethers::core::types::Address,
            amount: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([35, 184, 114, 221], (holder, recipient, amount))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Gets the contract's `Approval` event"]
        pub fn approval_filter(&self) -> ethers::contract::builders::Event<M, ApprovalFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `AuthorizedOperator` event"]
        pub fn authorized_operator_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, AuthorizedOperatorFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `Burned` event"]
        pub fn burned_filter(&self) -> ethers::contract::builders::Event<M, BurnedFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `Minted` event"]
        pub fn minted_filter(&self) -> ethers::contract::builders::Event<M, MintedFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `RevokedOperator` event"]
        pub fn revoked_operator_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, RevokedOperatorFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `Sent` event"]
        pub fn sent_filter(&self) -> ethers::contract::builders::Event<M, SentFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `Transfer` event"]
        pub fn transfer_filter(&self) -> ethers::contract::builders::Event<M, TransferFilter> {
            self.0.event()
        }
        #[doc = r" Returns an [`Event`](#ethers_contract::builders::Event) builder for all events of this contract"]
        pub fn events(&self) -> ethers::contract::builders::Event<M, ERC777SnapshotEvents> {
            self.0.event_with_filter(Default::default())
        }
    }
    impl<M: ethers::providers::Middleware> From<ethers::contract::Contract<M>> for ERC777Snapshot<M> {
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
    #[ethevent(
        name = "AuthorizedOperator",
        abi = "AuthorizedOperator(address,address)"
    )]
    pub struct AuthorizedOperatorFilter {
        #[ethevent(indexed)]
        pub operator: ethers::core::types::Address,
        #[ethevent(indexed)]
        pub token_holder: ethers::core::types::Address,
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
    #[ethevent(name = "Burned", abi = "Burned(address,address,uint256,bytes,bytes)")]
    pub struct BurnedFilter {
        #[ethevent(indexed)]
        pub operator: ethers::core::types::Address,
        #[ethevent(indexed)]
        pub from: ethers::core::types::Address,
        pub amount: ethers::core::types::U256,
        pub data: ethers::core::types::Bytes,
        pub operator_data: ethers::core::types::Bytes,
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
    #[ethevent(name = "Minted", abi = "Minted(address,address,uint256,bytes,bytes)")]
    pub struct MintedFilter {
        #[ethevent(indexed)]
        pub operator: ethers::core::types::Address,
        #[ethevent(indexed)]
        pub to: ethers::core::types::Address,
        pub amount: ethers::core::types::U256,
        pub data: ethers::core::types::Bytes,
        pub operator_data: ethers::core::types::Bytes,
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
    #[ethevent(name = "RevokedOperator", abi = "RevokedOperator(address,address)")]
    pub struct RevokedOperatorFilter {
        #[ethevent(indexed)]
        pub operator: ethers::core::types::Address,
        #[ethevent(indexed)]
        pub token_holder: ethers::core::types::Address,
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
        name = "Sent",
        abi = "Sent(address,address,address,uint256,bytes,bytes)"
    )]
    pub struct SentFilter {
        #[ethevent(indexed)]
        pub operator: ethers::core::types::Address,
        #[ethevent(indexed)]
        pub from: ethers::core::types::Address,
        #[ethevent(indexed)]
        pub to: ethers::core::types::Address,
        pub amount: ethers::core::types::U256,
        pub data: ethers::core::types::Bytes,
        pub operator_data: ethers::core::types::Bytes,
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
    pub struct TransferFilter {
        #[ethevent(indexed)]
        pub from: ethers::core::types::Address,
        #[ethevent(indexed)]
        pub to: ethers::core::types::Address,
        pub value: ethers::core::types::U256,
    }
    #[derive(Debug, Clone, PartialEq, Eq, ethers :: contract :: EthAbiType)]
    pub enum ERC777SnapshotEvents {
        ApprovalFilter(ApprovalFilter),
        AuthorizedOperatorFilter(AuthorizedOperatorFilter),
        BurnedFilter(BurnedFilter),
        MintedFilter(MintedFilter),
        RevokedOperatorFilter(RevokedOperatorFilter),
        SentFilter(SentFilter),
        TransferFilter(TransferFilter),
    }
    impl ethers::contract::EthLogDecode for ERC777SnapshotEvents {
        fn decode_log(
            log: &ethers::core::abi::RawLog,
        ) -> ::std::result::Result<Self, ethers::core::abi::Error>
        where
            Self: Sized,
        {
            if let Ok(decoded) = ApprovalFilter::decode_log(log) {
                return Ok(ERC777SnapshotEvents::ApprovalFilter(decoded));
            }
            if let Ok(decoded) = AuthorizedOperatorFilter::decode_log(log) {
                return Ok(ERC777SnapshotEvents::AuthorizedOperatorFilter(decoded));
            }
            if let Ok(decoded) = BurnedFilter::decode_log(log) {
                return Ok(ERC777SnapshotEvents::BurnedFilter(decoded));
            }
            if let Ok(decoded) = MintedFilter::decode_log(log) {
                return Ok(ERC777SnapshotEvents::MintedFilter(decoded));
            }
            if let Ok(decoded) = RevokedOperatorFilter::decode_log(log) {
                return Ok(ERC777SnapshotEvents::RevokedOperatorFilter(decoded));
            }
            if let Ok(decoded) = SentFilter::decode_log(log) {
                return Ok(ERC777SnapshotEvents::SentFilter(decoded));
            }
            if let Ok(decoded) = TransferFilter::decode_log(log) {
                return Ok(ERC777SnapshotEvents::TransferFilter(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData)
        }
    }
    impl ::std::fmt::Display for ERC777SnapshotEvents {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                ERC777SnapshotEvents::ApprovalFilter(element) => element.fmt(f),
                ERC777SnapshotEvents::AuthorizedOperatorFilter(element) => element.fmt(f),
                ERC777SnapshotEvents::BurnedFilter(element) => element.fmt(f),
                ERC777SnapshotEvents::MintedFilter(element) => element.fmt(f),
                ERC777SnapshotEvents::RevokedOperatorFilter(element) => element.fmt(f),
                ERC777SnapshotEvents::SentFilter(element) => element.fmt(f),
                ERC777SnapshotEvents::TransferFilter(element) => element.fmt(f),
            }
        }
    }
    #[doc = "Container type for all input parameters for the `accountSnapshots` function with signature `accountSnapshots(address,uint256)` and selector `[36, 151, 174, 230]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "accountSnapshots", abi = "accountSnapshots(address,uint256)")]
    pub struct AccountSnapshotsCall(
        pub ethers::core::types::Address,
        pub ethers::core::types::U256,
    );
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
        pub holder: ethers::core::types::Address,
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
    #[doc = "Container type for all input parameters for the `authorizeOperator` function with signature `authorizeOperator(address)` and selector `[149, 155, 140, 63]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "authorizeOperator", abi = "authorizeOperator(address)")]
    pub struct AuthorizeOperatorCall {
        pub operator: ethers::core::types::Address,
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
        pub token_holder: ethers::core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `balanceOfAt` function with signature `balanceOfAt(address,uint128)` and selector `[247, 114, 160, 146]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "balanceOfAt", abi = "balanceOfAt(address,uint128)")]
    pub struct BalanceOfAtCall {
        pub owner: ethers::core::types::Address,
        pub block_number: u128,
    }
    #[doc = "Container type for all input parameters for the `burn` function with signature `burn(uint256,bytes)` and selector `[254, 157, 147, 3]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "burn", abi = "burn(uint256,bytes)")]
    pub struct BurnCall {
        pub amount: ethers::core::types::U256,
        pub data: ethers::core::types::Bytes,
    }
    #[doc = "Container type for all input parameters for the `decimals` function with signature `decimals()` and selector `[49, 60, 229, 103]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "decimals", abi = "decimals()")]
    pub struct DecimalsCall;
    #[doc = "Container type for all input parameters for the `defaultOperators` function with signature `defaultOperators()` and selector `[6, 228, 133, 56]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "defaultOperators", abi = "defaultOperators()")]
    pub struct DefaultOperatorsCall;
    #[doc = "Container type for all input parameters for the `granularity` function with signature `granularity()` and selector `[85, 111, 13, 199]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "granularity", abi = "granularity()")]
    pub struct GranularityCall;
    #[doc = "Container type for all input parameters for the `isOperatorFor` function with signature `isOperatorFor(address,address)` and selector `[217, 91, 99, 113]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "isOperatorFor", abi = "isOperatorFor(address,address)")]
    pub struct IsOperatorForCall {
        pub operator: ethers::core::types::Address,
        pub token_holder: ethers::core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `name` function with signature `name()` and selector `[6, 253, 222, 3]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "name", abi = "name()")]
    pub struct NameCall;
    #[doc = "Container type for all input parameters for the `operatorBurn` function with signature `operatorBurn(address,uint256,bytes,bytes)` and selector `[252, 103, 60, 79]`"]
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
        name = "operatorBurn",
        abi = "operatorBurn(address,uint256,bytes,bytes)"
    )]
    pub struct OperatorBurnCall {
        pub account: ethers::core::types::Address,
        pub amount: ethers::core::types::U256,
        pub data: ethers::core::types::Bytes,
        pub operator_data: ethers::core::types::Bytes,
    }
    #[doc = "Container type for all input parameters for the `operatorSend` function with signature `operatorSend(address,address,uint256,bytes,bytes)` and selector `[98, 173, 27, 131]`"]
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
        name = "operatorSend",
        abi = "operatorSend(address,address,uint256,bytes,bytes)"
    )]
    pub struct OperatorSendCall {
        pub sender: ethers::core::types::Address,
        pub recipient: ethers::core::types::Address,
        pub amount: ethers::core::types::U256,
        pub data: ethers::core::types::Bytes,
        pub operator_data: ethers::core::types::Bytes,
    }
    #[doc = "Container type for all input parameters for the `revokeOperator` function with signature `revokeOperator(address)` and selector `[250, 216, 179, 42]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "revokeOperator", abi = "revokeOperator(address)")]
    pub struct RevokeOperatorCall {
        pub operator: ethers::core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `send` function with signature `send(address,uint256,bytes)` and selector `[155, 217, 187, 198]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "send", abi = "send(address,uint256,bytes)")]
    pub struct SendCall {
        pub recipient: ethers::core::types::Address,
        pub amount: ethers::core::types::U256,
        pub data: ethers::core::types::Bytes,
    }
    #[doc = "Container type for all input parameters for the `symbol` function with signature `symbol()` and selector `[149, 216, 155, 65]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "symbol", abi = "symbol()")]
    pub struct SymbolCall;
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
    #[doc = "Container type for all input parameters for the `totalSupplyAt` function with signature `totalSupplyAt(uint128)` and selector `[148, 121, 117, 217]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "totalSupplyAt", abi = "totalSupplyAt(uint128)")]
    pub struct TotalSupplyAtCall {
        pub block_number: u128,
    }
    #[doc = "Container type for all input parameters for the `totalSupplySnapshots` function with signature `totalSupplySnapshots(uint256)` and selector `[183, 215, 139, 26]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "totalSupplySnapshots", abi = "totalSupplySnapshots(uint256)")]
    pub struct TotalSupplySnapshotsCall(pub ethers::core::types::U256);
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
        pub recipient: ethers::core::types::Address,
        pub amount: ethers::core::types::U256,
    }
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
        pub holder: ethers::core::types::Address,
        pub recipient: ethers::core::types::Address,
        pub amount: ethers::core::types::U256,
    }
    #[derive(Debug, Clone, PartialEq, Eq, ethers :: contract :: EthAbiType)]
    pub enum ERC777SnapshotCalls {
        AccountSnapshots(AccountSnapshotsCall),
        Allowance(AllowanceCall),
        Approve(ApproveCall),
        AuthorizeOperator(AuthorizeOperatorCall),
        BalanceOf(BalanceOfCall),
        BalanceOfAt(BalanceOfAtCall),
        Burn(BurnCall),
        Decimals(DecimalsCall),
        DefaultOperators(DefaultOperatorsCall),
        Granularity(GranularityCall),
        IsOperatorFor(IsOperatorForCall),
        Name(NameCall),
        OperatorBurn(OperatorBurnCall),
        OperatorSend(OperatorSendCall),
        RevokeOperator(RevokeOperatorCall),
        Send(SendCall),
        Symbol(SymbolCall),
        TotalSupply(TotalSupplyCall),
        TotalSupplyAt(TotalSupplyAtCall),
        TotalSupplySnapshots(TotalSupplySnapshotsCall),
        Transfer(TransferCall),
        TransferFrom(TransferFromCall),
    }
    impl ethers::core::abi::AbiDecode for ERC777SnapshotCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::std::result::Result<Self, ethers::core::abi::AbiError> {
            if let Ok(decoded) =
                <AccountSnapshotsCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777SnapshotCalls::AccountSnapshots(decoded));
            }
            if let Ok(decoded) =
                <AllowanceCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777SnapshotCalls::Allowance(decoded));
            }
            if let Ok(decoded) =
                <ApproveCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777SnapshotCalls::Approve(decoded));
            }
            if let Ok(decoded) =
                <AuthorizeOperatorCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777SnapshotCalls::AuthorizeOperator(decoded));
            }
            if let Ok(decoded) =
                <BalanceOfCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777SnapshotCalls::BalanceOf(decoded));
            }
            if let Ok(decoded) =
                <BalanceOfAtCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777SnapshotCalls::BalanceOfAt(decoded));
            }
            if let Ok(decoded) = <BurnCall as ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(ERC777SnapshotCalls::Burn(decoded));
            }
            if let Ok(decoded) =
                <DecimalsCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777SnapshotCalls::Decimals(decoded));
            }
            if let Ok(decoded) =
                <DefaultOperatorsCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777SnapshotCalls::DefaultOperators(decoded));
            }
            if let Ok(decoded) =
                <GranularityCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777SnapshotCalls::Granularity(decoded));
            }
            if let Ok(decoded) =
                <IsOperatorForCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777SnapshotCalls::IsOperatorFor(decoded));
            }
            if let Ok(decoded) = <NameCall as ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(ERC777SnapshotCalls::Name(decoded));
            }
            if let Ok(decoded) =
                <OperatorBurnCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777SnapshotCalls::OperatorBurn(decoded));
            }
            if let Ok(decoded) =
                <OperatorSendCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777SnapshotCalls::OperatorSend(decoded));
            }
            if let Ok(decoded) =
                <RevokeOperatorCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777SnapshotCalls::RevokeOperator(decoded));
            }
            if let Ok(decoded) = <SendCall as ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(ERC777SnapshotCalls::Send(decoded));
            }
            if let Ok(decoded) = <SymbolCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777SnapshotCalls::Symbol(decoded));
            }
            if let Ok(decoded) =
                <TotalSupplyCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777SnapshotCalls::TotalSupply(decoded));
            }
            if let Ok(decoded) =
                <TotalSupplyAtCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777SnapshotCalls::TotalSupplyAt(decoded));
            }
            if let Ok(decoded) =
                <TotalSupplySnapshotsCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777SnapshotCalls::TotalSupplySnapshots(decoded));
            }
            if let Ok(decoded) =
                <TransferCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777SnapshotCalls::Transfer(decoded));
            }
            if let Ok(decoded) =
                <TransferFromCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777SnapshotCalls::TransferFrom(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ethers::core::abi::AbiEncode for ERC777SnapshotCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                ERC777SnapshotCalls::AccountSnapshots(element) => element.encode(),
                ERC777SnapshotCalls::Allowance(element) => element.encode(),
                ERC777SnapshotCalls::Approve(element) => element.encode(),
                ERC777SnapshotCalls::AuthorizeOperator(element) => element.encode(),
                ERC777SnapshotCalls::BalanceOf(element) => element.encode(),
                ERC777SnapshotCalls::BalanceOfAt(element) => element.encode(),
                ERC777SnapshotCalls::Burn(element) => element.encode(),
                ERC777SnapshotCalls::Decimals(element) => element.encode(),
                ERC777SnapshotCalls::DefaultOperators(element) => element.encode(),
                ERC777SnapshotCalls::Granularity(element) => element.encode(),
                ERC777SnapshotCalls::IsOperatorFor(element) => element.encode(),
                ERC777SnapshotCalls::Name(element) => element.encode(),
                ERC777SnapshotCalls::OperatorBurn(element) => element.encode(),
                ERC777SnapshotCalls::OperatorSend(element) => element.encode(),
                ERC777SnapshotCalls::RevokeOperator(element) => element.encode(),
                ERC777SnapshotCalls::Send(element) => element.encode(),
                ERC777SnapshotCalls::Symbol(element) => element.encode(),
                ERC777SnapshotCalls::TotalSupply(element) => element.encode(),
                ERC777SnapshotCalls::TotalSupplyAt(element) => element.encode(),
                ERC777SnapshotCalls::TotalSupplySnapshots(element) => element.encode(),
                ERC777SnapshotCalls::Transfer(element) => element.encode(),
                ERC777SnapshotCalls::TransferFrom(element) => element.encode(),
            }
        }
    }
    impl ::std::fmt::Display for ERC777SnapshotCalls {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                ERC777SnapshotCalls::AccountSnapshots(element) => element.fmt(f),
                ERC777SnapshotCalls::Allowance(element) => element.fmt(f),
                ERC777SnapshotCalls::Approve(element) => element.fmt(f),
                ERC777SnapshotCalls::AuthorizeOperator(element) => element.fmt(f),
                ERC777SnapshotCalls::BalanceOf(element) => element.fmt(f),
                ERC777SnapshotCalls::BalanceOfAt(element) => element.fmt(f),
                ERC777SnapshotCalls::Burn(element) => element.fmt(f),
                ERC777SnapshotCalls::Decimals(element) => element.fmt(f),
                ERC777SnapshotCalls::DefaultOperators(element) => element.fmt(f),
                ERC777SnapshotCalls::Granularity(element) => element.fmt(f),
                ERC777SnapshotCalls::IsOperatorFor(element) => element.fmt(f),
                ERC777SnapshotCalls::Name(element) => element.fmt(f),
                ERC777SnapshotCalls::OperatorBurn(element) => element.fmt(f),
                ERC777SnapshotCalls::OperatorSend(element) => element.fmt(f),
                ERC777SnapshotCalls::RevokeOperator(element) => element.fmt(f),
                ERC777SnapshotCalls::Send(element) => element.fmt(f),
                ERC777SnapshotCalls::Symbol(element) => element.fmt(f),
                ERC777SnapshotCalls::TotalSupply(element) => element.fmt(f),
                ERC777SnapshotCalls::TotalSupplyAt(element) => element.fmt(f),
                ERC777SnapshotCalls::TotalSupplySnapshots(element) => element.fmt(f),
                ERC777SnapshotCalls::Transfer(element) => element.fmt(f),
                ERC777SnapshotCalls::TransferFrom(element) => element.fmt(f),
            }
        }
    }
    impl ::std::convert::From<AccountSnapshotsCall> for ERC777SnapshotCalls {
        fn from(var: AccountSnapshotsCall) -> Self {
            ERC777SnapshotCalls::AccountSnapshots(var)
        }
    }
    impl ::std::convert::From<AllowanceCall> for ERC777SnapshotCalls {
        fn from(var: AllowanceCall) -> Self {
            ERC777SnapshotCalls::Allowance(var)
        }
    }
    impl ::std::convert::From<ApproveCall> for ERC777SnapshotCalls {
        fn from(var: ApproveCall) -> Self {
            ERC777SnapshotCalls::Approve(var)
        }
    }
    impl ::std::convert::From<AuthorizeOperatorCall> for ERC777SnapshotCalls {
        fn from(var: AuthorizeOperatorCall) -> Self {
            ERC777SnapshotCalls::AuthorizeOperator(var)
        }
    }
    impl ::std::convert::From<BalanceOfCall> for ERC777SnapshotCalls {
        fn from(var: BalanceOfCall) -> Self {
            ERC777SnapshotCalls::BalanceOf(var)
        }
    }
    impl ::std::convert::From<BalanceOfAtCall> for ERC777SnapshotCalls {
        fn from(var: BalanceOfAtCall) -> Self {
            ERC777SnapshotCalls::BalanceOfAt(var)
        }
    }
    impl ::std::convert::From<BurnCall> for ERC777SnapshotCalls {
        fn from(var: BurnCall) -> Self {
            ERC777SnapshotCalls::Burn(var)
        }
    }
    impl ::std::convert::From<DecimalsCall> for ERC777SnapshotCalls {
        fn from(var: DecimalsCall) -> Self {
            ERC777SnapshotCalls::Decimals(var)
        }
    }
    impl ::std::convert::From<DefaultOperatorsCall> for ERC777SnapshotCalls {
        fn from(var: DefaultOperatorsCall) -> Self {
            ERC777SnapshotCalls::DefaultOperators(var)
        }
    }
    impl ::std::convert::From<GranularityCall> for ERC777SnapshotCalls {
        fn from(var: GranularityCall) -> Self {
            ERC777SnapshotCalls::Granularity(var)
        }
    }
    impl ::std::convert::From<IsOperatorForCall> for ERC777SnapshotCalls {
        fn from(var: IsOperatorForCall) -> Self {
            ERC777SnapshotCalls::IsOperatorFor(var)
        }
    }
    impl ::std::convert::From<NameCall> for ERC777SnapshotCalls {
        fn from(var: NameCall) -> Self {
            ERC777SnapshotCalls::Name(var)
        }
    }
    impl ::std::convert::From<OperatorBurnCall> for ERC777SnapshotCalls {
        fn from(var: OperatorBurnCall) -> Self {
            ERC777SnapshotCalls::OperatorBurn(var)
        }
    }
    impl ::std::convert::From<OperatorSendCall> for ERC777SnapshotCalls {
        fn from(var: OperatorSendCall) -> Self {
            ERC777SnapshotCalls::OperatorSend(var)
        }
    }
    impl ::std::convert::From<RevokeOperatorCall> for ERC777SnapshotCalls {
        fn from(var: RevokeOperatorCall) -> Self {
            ERC777SnapshotCalls::RevokeOperator(var)
        }
    }
    impl ::std::convert::From<SendCall> for ERC777SnapshotCalls {
        fn from(var: SendCall) -> Self {
            ERC777SnapshotCalls::Send(var)
        }
    }
    impl ::std::convert::From<SymbolCall> for ERC777SnapshotCalls {
        fn from(var: SymbolCall) -> Self {
            ERC777SnapshotCalls::Symbol(var)
        }
    }
    impl ::std::convert::From<TotalSupplyCall> for ERC777SnapshotCalls {
        fn from(var: TotalSupplyCall) -> Self {
            ERC777SnapshotCalls::TotalSupply(var)
        }
    }
    impl ::std::convert::From<TotalSupplyAtCall> for ERC777SnapshotCalls {
        fn from(var: TotalSupplyAtCall) -> Self {
            ERC777SnapshotCalls::TotalSupplyAt(var)
        }
    }
    impl ::std::convert::From<TotalSupplySnapshotsCall> for ERC777SnapshotCalls {
        fn from(var: TotalSupplySnapshotsCall) -> Self {
            ERC777SnapshotCalls::TotalSupplySnapshots(var)
        }
    }
    impl ::std::convert::From<TransferCall> for ERC777SnapshotCalls {
        fn from(var: TransferCall) -> Self {
            ERC777SnapshotCalls::Transfer(var)
        }
    }
    impl ::std::convert::From<TransferFromCall> for ERC777SnapshotCalls {
        fn from(var: TransferFromCall) -> Self {
            ERC777SnapshotCalls::TransferFrom(var)
        }
    }
    #[doc = "Container type for all return fields from the `accountSnapshots` function with signature `accountSnapshots(address,uint256)` and selector `[36, 151, 174, 230]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct AccountSnapshotsReturn {
        pub from_block: u128,
        pub value: u128,
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
    #[doc = "Container type for all return fields from the `balanceOfAt` function with signature `balanceOfAt(address,uint128)` and selector `[247, 114, 160, 146]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct BalanceOfAtReturn(pub ethers::core::types::U256);
    #[doc = "Container type for all return fields from the `decimals` function with signature `decimals()` and selector `[49, 60, 229, 103]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct DecimalsReturn(pub u8);
    #[doc = "Container type for all return fields from the `defaultOperators` function with signature `defaultOperators()` and selector `[6, 228, 133, 56]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct DefaultOperatorsReturn(pub ::std::vec::Vec<ethers::core::types::Address>);
    #[doc = "Container type for all return fields from the `granularity` function with signature `granularity()` and selector `[85, 111, 13, 199]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct GranularityReturn(pub ethers::core::types::U256);
    #[doc = "Container type for all return fields from the `isOperatorFor` function with signature `isOperatorFor(address,address)` and selector `[217, 91, 99, 113]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct IsOperatorForReturn(pub bool);
    #[doc = "Container type for all return fields from the `name` function with signature `name()` and selector `[6, 253, 222, 3]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct NameReturn(pub String);
    #[doc = "Container type for all return fields from the `symbol` function with signature `symbol()` and selector `[149, 216, 155, 65]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct SymbolReturn(pub String);
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
    #[doc = "Container type for all return fields from the `totalSupplyAt` function with signature `totalSupplyAt(uint128)` and selector `[148, 121, 117, 217]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct TotalSupplyAtReturn(pub ethers::core::types::U256);
    #[doc = "Container type for all return fields from the `totalSupplySnapshots` function with signature `totalSupplySnapshots(uint256)` and selector `[183, 215, 139, 26]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct TotalSupplySnapshotsReturn {
        pub from_block: u128,
        pub value: u128,
    }
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
