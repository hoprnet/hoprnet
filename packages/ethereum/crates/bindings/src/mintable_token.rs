pub use mintable_token::*;
#[allow(clippy::too_many_arguments, non_camel_case_types)]
pub mod mintable_token {
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
    #[doc = "MintableToken was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    # [rustfmt :: skip] const __ABI : & str = "[{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"mintingFinished\",\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"name\":\"_spender\",\"type\":\"address\",\"components\":[]},{\"name\":\"_value\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"approve\",\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"totalSupply\",\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"name\":\"_from\",\"type\":\"address\",\"components\":[]},{\"name\":\"_to\",\"type\":\"address\",\"components\":[]},{\"name\":\"_value\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"transferFrom\",\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"name\":\"_to\",\"type\":\"address\",\"components\":[]},{\"name\":\"_amount\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"mint\",\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"name\":\"_spender\",\"type\":\"address\",\"components\":[]},{\"name\":\"_subtractedValue\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"decreaseApproval\",\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"name\":\"_owner\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"balanceOf\",\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"renounceOwnership\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"finishMinting\",\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"owner\",\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[{\"name\":\"_to\",\"type\":\"address\",\"components\":[]},{\"name\":\"_value\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"transfer\",\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"name\":\"_spender\",\"type\":\"address\",\"components\":[]},{\"name\":\"_addedValue\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"increaseApproval\",\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"name\":\"_owner\",\"type\":\"address\",\"components\":[]},{\"name\":\"_spender\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"allowance\",\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"name\":\"_newOwner\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"transferOwnership\",\"outputs\":[]},{\"inputs\":[{\"name\":\"to\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"name\":\"amount\",\"type\":\"uint256\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"Mint\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[],\"type\":\"event\",\"name\":\"MintFinished\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"name\":\"previousOwner\",\"type\":\"address\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"OwnershipRenounced\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"name\":\"previousOwner\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"name\":\"newOwner\",\"type\":\"address\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"OwnershipTransferred\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"name\":\"owner\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"name\":\"spender\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"name\":\"value\",\"type\":\"uint256\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"Approval\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"name\":\"from\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"name\":\"to\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"name\":\"value\",\"type\":\"uint256\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"Transfer\",\"outputs\":[],\"anonymous\":false}]" ;
    #[doc = r" The parsed JSON-ABI of the contract."]
    pub static MINTABLETOKEN_ABI: ethers::contract::Lazy<ethers::core::abi::Abi> =
        ethers::contract::Lazy::new(|| {
            ethers::core::utils::__serde_json::from_str(__ABI).expect("invalid abi")
        });
    #[doc = r" Bytecode of the #name contract"]
    pub static MINTABLETOKEN_BYTECODE: ethers::contract::Lazy<ethers::core::types::Bytes> =
        ethers::contract::Lazy::new(|| {
            "0x608060405260038054600160a860020a03191633179055610a8b806100256000396000f3006080604052600436106100b35760003560e01c63ffffffff16806305d2035b146100b8578063095ea7b3146100e157806318160ddd1461010557806323b872dd1461012c57806340c10f1914610156578063661884631461017a57806370a082311461019e578063715018a6146101bf5780637d64bcb4146101d65780638da5cb5b146101eb578063a9059cbb1461021c578063d73dd62314610240578063dd62ed3e14610264578063f2fde38b1461028b575b600080fd5b3480156100c457600080fd5b506100cd6102ac565b604080519115158252519081900360200190f35b3480156100ed57600080fd5b506100cd600160a060020a03600435166024356102cd565b34801561011157600080fd5b5061011a610333565b60408051918252519081900360200190f35b34801561013857600080fd5b506100cd600160a060020a0360043581169060243516604435610339565b34801561016257600080fd5b506100cd600160a060020a03600435166024356104ae565b34801561018657600080fd5b506100cd600160a060020a03600435166024356105c9565b3480156101aa57600080fd5b5061011a600160a060020a03600435166106b8565b3480156101cb57600080fd5b506101d46106d3565b005b3480156101e257600080fd5b506100cd610741565b3480156101f757600080fd5b506102006107e7565b60408051600160a060020a039092168252519081900360200190f35b34801561022857600080fd5b506100cd600160a060020a03600435166024356107f6565b34801561024c57600080fd5b506100cd600160a060020a03600435166024356108d5565b34801561027057600080fd5b5061011a600160a060020a036004358116906024351661096e565b34801561029757600080fd5b506101d4600160a060020a0360043516610999565b60035474010000000000000000000000000000000000000000900460ff1681565b336000818152600260209081526040808320600160a060020a038716808552908352818420869055815186815291519394909390927f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925928290030190a350600192915050565b60015490565b600160a060020a03831660009081526020819052604081205482111561035e57600080fd5b600160a060020a038416600090815260026020908152604080832033845290915290205482111561038e57600080fd5b600160a060020a03831615156103a357600080fd5b600160a060020a0384166000908152602081905260409020546103cc908363ffffffff6109bc16565b600160a060020a038086166000908152602081905260408082209390935590851681522054610401908363ffffffff6109ce16565b600160a060020a03808516600090815260208181526040808320949094559187168152600282528281203382529091522054610443908363ffffffff6109bc16565b600160a060020a03808616600081815260026020908152604080832033845282529182902094909455805186815290519287169391927fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef929181900390910190a35060019392505050565b600354600090600160a060020a031633146104c857600080fd5b60035474010000000000000000000000000000000000000000900460ff16156104f057600080fd5b600154610503908363ffffffff6109ce16565b600155600160a060020a03831660009081526020819052604090205461052f908363ffffffff6109ce16565b600160a060020a03841660008181526020818152604091829020939093558051858152905191927f0f6798a560793a54c3bcfe86a93cde1e73087d944c0ea20544137d412139688592918290030190a2604080518381529051600160a060020a038516916000917fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef9181900360200190a350600192915050565b336000908152600260209081526040808320600160a060020a038616845290915281205480831061061d57336000908152600260209081526040808320600160a060020a0388168452909152812055610652565b61062d818463ffffffff6109bc16565b336000908152600260209081526040808320600160a060020a03891684529091529020555b336000818152600260209081526040808320600160a060020a0389168085529083529281902054815190815290519293927f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925929181900390910190a35060019392505050565b600160a060020a031660009081526020819052604090205490565b600354600160a060020a031633146106ea57600080fd5b600354604051600160a060020a03909116907ff8df31144d9c2f0f6b59d69b8b98abd5459d07f2742c4df920b25aae33c6482090600090a26003805473ffffffffffffffffffffffffffffffffffffffff19169055565b600354600090600160a060020a0316331461075b57600080fd5b60035474010000000000000000000000000000000000000000900460ff161561078357600080fd5b6003805474ff00000000000000000000000000000000000000001916740100000000000000000000000000000000000000001790556040517fae5184fba832cb2b1f702aca6117b8d265eaf03ad33eb133f19dde0f5920fa0890600090a150600190565b600354600160a060020a031681565b3360009081526020819052604081205482111561081257600080fd5b600160a060020a038316151561082757600080fd5b33600090815260208190526040902054610847908363ffffffff6109bc16565b3360009081526020819052604080822092909255600160a060020a03851681522054610879908363ffffffff6109ce16565b600160a060020a038416600081815260208181526040918290209390935580518581529051919233927fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef9281900390910190a350600192915050565b336000908152600260209081526040808320600160a060020a0386168452909152812054610909908363ffffffff6109ce16565b336000818152600260209081526040808320600160a060020a0389168085529083529281902085905580519485525191937f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925929081900390910190a350600192915050565b600160a060020a03918216600090815260026020908152604080832093909416825291909152205490565b600354600160a060020a031633146109b057600080fd5b6109b9816109e1565b50565b6000828211156109c857fe5b50900390565b818101828110156109db57fe5b92915050565b600160a060020a03811615156109f657600080fd5b600354604051600160a060020a038084169216907f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e090600090a36003805473ffffffffffffffffffffffffffffffffffffffff1916600160a060020a03929092169190911790555600a165627a7a72305820c95122540059fded105328e2d471a5efea8a2ada1bff92be1f4411c64c1e52f30029" . parse () . expect ("invalid bytecode")
        });
    pub struct MintableToken<M>(ethers::contract::Contract<M>);
    impl<M> Clone for MintableToken<M> {
        fn clone(&self) -> Self {
            MintableToken(self.0.clone())
        }
    }
    impl<M> std::ops::Deref for MintableToken<M> {
        type Target = ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> std::fmt::Debug for MintableToken<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(MintableToken))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ethers::providers::Middleware> MintableToken<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            ethers::contract::Contract::new(address.into(), MINTABLETOKEN_ABI.clone(), client)
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
                MINTABLETOKEN_ABI.clone(),
                MINTABLETOKEN_BYTECODE.clone().into(),
                client,
            );
            let deployer = factory.deploy(constructor_args)?;
            let deployer = ethers::contract::ContractDeployer::new(deployer);
            Ok(deployer)
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
            owner: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([112, 160, 130, 49], owner)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `decreaseApproval` (0x66188463) function"]
        pub fn decrease_approval(
            &self,
            spender: ethers::core::types::Address,
            subtracted_value: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([102, 24, 132, 99], (spender, subtracted_value))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `finishMinting` (0x7d64bcb4) function"]
        pub fn finish_minting(&self) -> ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([125, 100, 188, 180], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `increaseApproval` (0xd73dd623) function"]
        pub fn increase_approval(
            &self,
            spender: ethers::core::types::Address,
            added_value: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([215, 61, 214, 35], (spender, added_value))
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
        #[doc = "Calls the contract's `mintingFinished` (0x05d2035b) function"]
        pub fn minting_finished(&self) -> ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([5, 210, 3, 91], ())
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
        #[doc = "Calls the contract's `renounceOwnership` (0x715018a6) function"]
        pub fn renounce_ownership(&self) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([113, 80, 24, 166], ())
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
        #[doc = "Calls the contract's `transferOwnership` (0xf2fde38b) function"]
        pub fn transfer_ownership(
            &self,
            new_owner: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([242, 253, 227, 139], new_owner)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Gets the contract's `Approval` event"]
        pub fn approval_filter(&self) -> ethers::contract::builders::Event<M, ApprovalFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `Mint` event"]
        pub fn mint_filter(&self) -> ethers::contract::builders::Event<M, MintFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `MintFinished` event"]
        pub fn mint_finished_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, MintFinishedFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `OwnershipRenounced` event"]
        pub fn ownership_renounced_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, OwnershipRenouncedFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `OwnershipTransferred` event"]
        pub fn ownership_transferred_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, OwnershipTransferredFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `Transfer` event"]
        pub fn transfer_filter(&self) -> ethers::contract::builders::Event<M, TransferFilter> {
            self.0.event()
        }
        #[doc = r" Returns an [`Event`](#ethers_contract::builders::Event) builder for all events of this contract"]
        pub fn events(&self) -> ethers::contract::builders::Event<M, MintableTokenEvents> {
            self.0.event_with_filter(Default::default())
        }
    }
    impl<M: ethers::providers::Middleware> From<ethers::contract::Contract<M>> for MintableToken<M> {
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
    #[ethevent(name = "Mint", abi = "Mint(address,uint256)")]
    pub struct MintFilter {
        #[ethevent(indexed)]
        pub to: ethers::core::types::Address,
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
    #[ethevent(name = "MintFinished", abi = "MintFinished()")]
    pub struct MintFinishedFilter();
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthEvent,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethevent(name = "OwnershipRenounced", abi = "OwnershipRenounced(address)")]
    pub struct OwnershipRenouncedFilter {
        #[ethevent(indexed)]
        pub previous_owner: ethers::core::types::Address,
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
    #[ethevent(name = "Transfer", abi = "Transfer(address,address,uint256)")]
    pub struct TransferFilter {
        #[ethevent(indexed)]
        pub from: ethers::core::types::Address,
        #[ethevent(indexed)]
        pub to: ethers::core::types::Address,
        pub value: ethers::core::types::U256,
    }
    #[derive(Debug, Clone, PartialEq, Eq, ethers :: contract :: EthAbiType)]
    pub enum MintableTokenEvents {
        ApprovalFilter(ApprovalFilter),
        MintFilter(MintFilter),
        MintFinishedFilter(MintFinishedFilter),
        OwnershipRenouncedFilter(OwnershipRenouncedFilter),
        OwnershipTransferredFilter(OwnershipTransferredFilter),
        TransferFilter(TransferFilter),
    }
    impl ethers::contract::EthLogDecode for MintableTokenEvents {
        fn decode_log(
            log: &ethers::core::abi::RawLog,
        ) -> ::std::result::Result<Self, ethers::core::abi::Error>
        where
            Self: Sized,
        {
            if let Ok(decoded) = ApprovalFilter::decode_log(log) {
                return Ok(MintableTokenEvents::ApprovalFilter(decoded));
            }
            if let Ok(decoded) = MintFilter::decode_log(log) {
                return Ok(MintableTokenEvents::MintFilter(decoded));
            }
            if let Ok(decoded) = MintFinishedFilter::decode_log(log) {
                return Ok(MintableTokenEvents::MintFinishedFilter(decoded));
            }
            if let Ok(decoded) = OwnershipRenouncedFilter::decode_log(log) {
                return Ok(MintableTokenEvents::OwnershipRenouncedFilter(decoded));
            }
            if let Ok(decoded) = OwnershipTransferredFilter::decode_log(log) {
                return Ok(MintableTokenEvents::OwnershipTransferredFilter(decoded));
            }
            if let Ok(decoded) = TransferFilter::decode_log(log) {
                return Ok(MintableTokenEvents::TransferFilter(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData)
        }
    }
    impl ::std::fmt::Display for MintableTokenEvents {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                MintableTokenEvents::ApprovalFilter(element) => element.fmt(f),
                MintableTokenEvents::MintFilter(element) => element.fmt(f),
                MintableTokenEvents::MintFinishedFilter(element) => element.fmt(f),
                MintableTokenEvents::OwnershipRenouncedFilter(element) => element.fmt(f),
                MintableTokenEvents::OwnershipTransferredFilter(element) => element.fmt(f),
                MintableTokenEvents::TransferFilter(element) => element.fmt(f),
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
        pub owner: ethers::core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `decreaseApproval` function with signature `decreaseApproval(address,uint256)` and selector `[102, 24, 132, 99]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "decreaseApproval", abi = "decreaseApproval(address,uint256)")]
    pub struct DecreaseApprovalCall {
        pub spender: ethers::core::types::Address,
        pub subtracted_value: ethers::core::types::U256,
    }
    #[doc = "Container type for all input parameters for the `finishMinting` function with signature `finishMinting()` and selector `[125, 100, 188, 180]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "finishMinting", abi = "finishMinting()")]
    pub struct FinishMintingCall;
    #[doc = "Container type for all input parameters for the `increaseApproval` function with signature `increaseApproval(address,uint256)` and selector `[215, 61, 214, 35]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "increaseApproval", abi = "increaseApproval(address,uint256)")]
    pub struct IncreaseApprovalCall {
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
    #[doc = "Container type for all input parameters for the `mintingFinished` function with signature `mintingFinished()` and selector `[5, 210, 3, 91]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "mintingFinished", abi = "mintingFinished()")]
    pub struct MintingFinishedCall;
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
    #[derive(Debug, Clone, PartialEq, Eq, ethers :: contract :: EthAbiType)]
    pub enum MintableTokenCalls {
        Allowance(AllowanceCall),
        Approve(ApproveCall),
        BalanceOf(BalanceOfCall),
        DecreaseApproval(DecreaseApprovalCall),
        FinishMinting(FinishMintingCall),
        IncreaseApproval(IncreaseApprovalCall),
        Mint(MintCall),
        MintingFinished(MintingFinishedCall),
        Owner(OwnerCall),
        RenounceOwnership(RenounceOwnershipCall),
        TotalSupply(TotalSupplyCall),
        Transfer(TransferCall),
        TransferFrom(TransferFromCall),
        TransferOwnership(TransferOwnershipCall),
    }
    impl ethers::core::abi::AbiDecode for MintableTokenCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::std::result::Result<Self, ethers::core::abi::AbiError> {
            if let Ok(decoded) =
                <AllowanceCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(MintableTokenCalls::Allowance(decoded));
            }
            if let Ok(decoded) =
                <ApproveCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(MintableTokenCalls::Approve(decoded));
            }
            if let Ok(decoded) =
                <BalanceOfCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(MintableTokenCalls::BalanceOf(decoded));
            }
            if let Ok(decoded) =
                <DecreaseApprovalCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(MintableTokenCalls::DecreaseApproval(decoded));
            }
            if let Ok(decoded) =
                <FinishMintingCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(MintableTokenCalls::FinishMinting(decoded));
            }
            if let Ok(decoded) =
                <IncreaseApprovalCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(MintableTokenCalls::IncreaseApproval(decoded));
            }
            if let Ok(decoded) = <MintCall as ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(MintableTokenCalls::Mint(decoded));
            }
            if let Ok(decoded) =
                <MintingFinishedCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(MintableTokenCalls::MintingFinished(decoded));
            }
            if let Ok(decoded) = <OwnerCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(MintableTokenCalls::Owner(decoded));
            }
            if let Ok(decoded) =
                <RenounceOwnershipCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(MintableTokenCalls::RenounceOwnership(decoded));
            }
            if let Ok(decoded) =
                <TotalSupplyCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(MintableTokenCalls::TotalSupply(decoded));
            }
            if let Ok(decoded) =
                <TransferCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(MintableTokenCalls::Transfer(decoded));
            }
            if let Ok(decoded) =
                <TransferFromCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(MintableTokenCalls::TransferFrom(decoded));
            }
            if let Ok(decoded) =
                <TransferOwnershipCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(MintableTokenCalls::TransferOwnership(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ethers::core::abi::AbiEncode for MintableTokenCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                MintableTokenCalls::Allowance(element) => element.encode(),
                MintableTokenCalls::Approve(element) => element.encode(),
                MintableTokenCalls::BalanceOf(element) => element.encode(),
                MintableTokenCalls::DecreaseApproval(element) => element.encode(),
                MintableTokenCalls::FinishMinting(element) => element.encode(),
                MintableTokenCalls::IncreaseApproval(element) => element.encode(),
                MintableTokenCalls::Mint(element) => element.encode(),
                MintableTokenCalls::MintingFinished(element) => element.encode(),
                MintableTokenCalls::Owner(element) => element.encode(),
                MintableTokenCalls::RenounceOwnership(element) => element.encode(),
                MintableTokenCalls::TotalSupply(element) => element.encode(),
                MintableTokenCalls::Transfer(element) => element.encode(),
                MintableTokenCalls::TransferFrom(element) => element.encode(),
                MintableTokenCalls::TransferOwnership(element) => element.encode(),
            }
        }
    }
    impl ::std::fmt::Display for MintableTokenCalls {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                MintableTokenCalls::Allowance(element) => element.fmt(f),
                MintableTokenCalls::Approve(element) => element.fmt(f),
                MintableTokenCalls::BalanceOf(element) => element.fmt(f),
                MintableTokenCalls::DecreaseApproval(element) => element.fmt(f),
                MintableTokenCalls::FinishMinting(element) => element.fmt(f),
                MintableTokenCalls::IncreaseApproval(element) => element.fmt(f),
                MintableTokenCalls::Mint(element) => element.fmt(f),
                MintableTokenCalls::MintingFinished(element) => element.fmt(f),
                MintableTokenCalls::Owner(element) => element.fmt(f),
                MintableTokenCalls::RenounceOwnership(element) => element.fmt(f),
                MintableTokenCalls::TotalSupply(element) => element.fmt(f),
                MintableTokenCalls::Transfer(element) => element.fmt(f),
                MintableTokenCalls::TransferFrom(element) => element.fmt(f),
                MintableTokenCalls::TransferOwnership(element) => element.fmt(f),
            }
        }
    }
    impl ::std::convert::From<AllowanceCall> for MintableTokenCalls {
        fn from(var: AllowanceCall) -> Self {
            MintableTokenCalls::Allowance(var)
        }
    }
    impl ::std::convert::From<ApproveCall> for MintableTokenCalls {
        fn from(var: ApproveCall) -> Self {
            MintableTokenCalls::Approve(var)
        }
    }
    impl ::std::convert::From<BalanceOfCall> for MintableTokenCalls {
        fn from(var: BalanceOfCall) -> Self {
            MintableTokenCalls::BalanceOf(var)
        }
    }
    impl ::std::convert::From<DecreaseApprovalCall> for MintableTokenCalls {
        fn from(var: DecreaseApprovalCall) -> Self {
            MintableTokenCalls::DecreaseApproval(var)
        }
    }
    impl ::std::convert::From<FinishMintingCall> for MintableTokenCalls {
        fn from(var: FinishMintingCall) -> Self {
            MintableTokenCalls::FinishMinting(var)
        }
    }
    impl ::std::convert::From<IncreaseApprovalCall> for MintableTokenCalls {
        fn from(var: IncreaseApprovalCall) -> Self {
            MintableTokenCalls::IncreaseApproval(var)
        }
    }
    impl ::std::convert::From<MintCall> for MintableTokenCalls {
        fn from(var: MintCall) -> Self {
            MintableTokenCalls::Mint(var)
        }
    }
    impl ::std::convert::From<MintingFinishedCall> for MintableTokenCalls {
        fn from(var: MintingFinishedCall) -> Self {
            MintableTokenCalls::MintingFinished(var)
        }
    }
    impl ::std::convert::From<OwnerCall> for MintableTokenCalls {
        fn from(var: OwnerCall) -> Self {
            MintableTokenCalls::Owner(var)
        }
    }
    impl ::std::convert::From<RenounceOwnershipCall> for MintableTokenCalls {
        fn from(var: RenounceOwnershipCall) -> Self {
            MintableTokenCalls::RenounceOwnership(var)
        }
    }
    impl ::std::convert::From<TotalSupplyCall> for MintableTokenCalls {
        fn from(var: TotalSupplyCall) -> Self {
            MintableTokenCalls::TotalSupply(var)
        }
    }
    impl ::std::convert::From<TransferCall> for MintableTokenCalls {
        fn from(var: TransferCall) -> Self {
            MintableTokenCalls::Transfer(var)
        }
    }
    impl ::std::convert::From<TransferFromCall> for MintableTokenCalls {
        fn from(var: TransferFromCall) -> Self {
            MintableTokenCalls::TransferFrom(var)
        }
    }
    impl ::std::convert::From<TransferOwnershipCall> for MintableTokenCalls {
        fn from(var: TransferOwnershipCall) -> Self {
            MintableTokenCalls::TransferOwnership(var)
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
    #[doc = "Container type for all return fields from the `decreaseApproval` function with signature `decreaseApproval(address,uint256)` and selector `[102, 24, 132, 99]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct DecreaseApprovalReturn(pub bool);
    #[doc = "Container type for all return fields from the `finishMinting` function with signature `finishMinting()` and selector `[125, 100, 188, 180]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct FinishMintingReturn(pub bool);
    #[doc = "Container type for all return fields from the `increaseApproval` function with signature `increaseApproval(address,uint256)` and selector `[215, 61, 214, 35]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct IncreaseApprovalReturn(pub bool);
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
    #[doc = "Container type for all return fields from the `mintingFinished` function with signature `mintingFinished()` and selector `[5, 210, 3, 91]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct MintingFinishedReturn(pub bool);
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
