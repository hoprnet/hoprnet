pub use erc777_sender_recipient_mock::*;
#[allow(clippy::too_many_arguments, non_camel_case_types)]
pub mod erc777_sender_recipient_mock {
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
    #[doc = "ERC777SenderRecipientMock was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    # [rustfmt :: skip] const __ABI : & str = "[{\"inputs\":[{\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\",\"components\":[],\"indexed\":false},{\"internalType\":\"address\",\"name\":\"from\",\"type\":\"address\",\"components\":[],\"indexed\":false},{\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\",\"components\":[],\"indexed\":false},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[],\"indexed\":false},{\"internalType\":\"bytes\",\"name\":\"data\",\"type\":\"bytes\",\"components\":[],\"indexed\":false},{\"internalType\":\"bytes\",\"name\":\"operatorData\",\"type\":\"bytes\",\"components\":[],\"indexed\":false},{\"internalType\":\"address\",\"name\":\"token\",\"type\":\"address\",\"components\":[],\"indexed\":false},{\"internalType\":\"uint256\",\"name\":\"fromBalance\",\"type\":\"uint256\",\"components\":[],\"indexed\":false},{\"internalType\":\"uint256\",\"name\":\"toBalance\",\"type\":\"uint256\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"TokensReceivedCalled\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\",\"components\":[],\"indexed\":false},{\"internalType\":\"address\",\"name\":\"from\",\"type\":\"address\",\"components\":[],\"indexed\":false},{\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\",\"components\":[],\"indexed\":false},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[],\"indexed\":false},{\"internalType\":\"bytes\",\"name\":\"data\",\"type\":\"bytes\",\"components\":[],\"indexed\":false},{\"internalType\":\"bytes\",\"name\":\"operatorData\",\"type\":\"bytes\",\"components\":[],\"indexed\":false},{\"internalType\":\"address\",\"name\":\"token\",\"type\":\"address\",\"components\":[],\"indexed\":false},{\"internalType\":\"uint256\",\"name\":\"fromBalance\",\"type\":\"uint256\",\"components\":[],\"indexed\":false},{\"internalType\":\"uint256\",\"name\":\"toBalance\",\"type\":\"uint256\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"TokensToSendCalled\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"contract IERC777\",\"name\":\"token\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"data\",\"type\":\"bytes\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"burn\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"interfaceHash\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"canImplementInterfaceForAddress\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"recipientFor\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"recipient\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"registerRecipient\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"sender\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"registerSender\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"contract IERC777\",\"name\":\"token\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"data\",\"type\":\"bytes\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"send\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"senderFor\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"bool\",\"name\":\"shouldRevert\",\"type\":\"bool\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"setShouldRevertReceive\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"bool\",\"name\":\"shouldRevert\",\"type\":\"bool\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"setShouldRevertSend\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"from\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"userData\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"operatorData\",\"type\":\"bytes\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"tokensReceived\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"from\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"userData\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"operatorData\",\"type\":\"bytes\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"tokensToSend\",\"outputs\":[]}]" ;
    #[doc = r" The parsed JSON-ABI of the contract."]
    pub static ERC777SENDERRECIPIENTMOCK_ABI: ethers::contract::Lazy<ethers::core::abi::Abi> =
        ethers::contract::Lazy::new(|| {
            ethers::core::utils::__serde_json::from_str(__ABI).expect("invalid abi")
        });
    #[doc = r" Bytecode of the #name contract"]
    pub static ERC777SENDERRECIPIENTMOCK_BYTECODE: ethers::contract::Lazy<
        ethers::core::types::Bytes,
    > = ethers::contract::Lazy::new(|| {
        "0x60806040526001805462010000600160b01b031916751820a4b7618bde71dce8cdc73aab6c95905fad24000017905534801561003a57600080fd5b50610da58061004a6000396000f3fe608060405234801561001057600080fd5b50600436106100a85760003560e01c806375ab97821161007157806375ab97821461036f578063a8badaa514610455578063c97e18fc1461047b578063d2de64741461049a578063e0eb2180146104c0578063e1ecbd30146104e6576100a8565b806223de29146100ad578063249cb3fa146101955780633836ef89146101d357806344d17187146102975780634e4ae5a514610350575b600080fd5b610193600480360360c08110156100c357600080fd5b6001600160a01b03823581169260208101358216926040820135909216916060820135919081019060a081016080820135600160201b81111561010557600080fd5b82018360208201111561011757600080fd5b803590602001918460018302840111600160201b8311171561013857600080fd5b919390929091602081019035600160201b81111561015557600080fd5b82018360208201111561016757600080fd5b803590602001918460018302840111600160201b8311171561018857600080fd5b50909250905061050c565b005b6101c1600480360360408110156101ab57600080fd5b50803590602001356001600160a01b031661072a565b60408051918252519081900360200190f35b610193600480360360808110156101e957600080fd5b6001600160a01b03823581169260208101359091169160408201359190810190608081016060820135600160201b81111561022357600080fd5b82018360208201111561023557600080fd5b803590602001918460018302840111600160201b8311171561025657600080fd5b91908080601f01602080910402602001604051908101604052809392919081815260200183838082843760009201919091525092955061079f945050505050565b610193600480360360608110156102ad57600080fd5b6001600160a01b0382351691602081013591810190606081016040820135600160201b8111156102dc57600080fd5b8201836020820111156102ee57600080fd5b803590602001918460018302840111600160201b8311171561030f57600080fd5b91908080601f01602080910402602001604051908101604052809392919081815260200183838082843760009201919091525092955061087e945050505050565b6101936004803603602081101561036657600080fd5b5035151561094c565b610193600480360360c081101561038557600080fd5b6001600160a01b03823581169260208101358216926040820135909216916060820135919081019060a081016080820135600160201b8111156103c757600080fd5b8201836020820111156103d957600080fd5b803590602001918460018302840111600160201b831117156103fa57600080fd5b919390929091602081019035600160201b81111561041757600080fd5b82018360208201111561042957600080fd5b803590602001918460018302840111600160201b8311171561044a57600080fd5b50909250905061095f565b6101936004803603602081101561046b57600080fd5b50356001600160a01b0316610b78565b6101936004803603602081101561049157600080fd5b50351515610c14565b610193600480360360208110156104b057600080fd5b50356001600160a01b0316610c2e565b610193600480360360208110156104d657600080fd5b50356001600160a01b0316610c77565b610193600480360360208110156104fc57600080fd5b50356001600160a01b0316610cbc565b600154610100900460ff161561052157600080fd5b600061052b610d3d565b90506000816001600160a01b03166370a082318a6040518263ffffffff1660e01b815260040180826001600160a01b0316815260200191505060206040518083038186803b15801561057c57600080fd5b505afa158015610590573d6000803e3d6000fd5b505050506040513d60208110156105a657600080fd5b5051604080516370a0823160e01b81526001600160a01b038b811660048301529151929350600092918516916370a0823191602480820192602092909190829003018186803b1580156105f857600080fd5b505afa15801561060c573d6000803e3d6000fd5b505050506040513d602081101561062257600080fd5b810190808051906020019092919050505090507f47e915878c47f3ec4d7ff646a2becb229f64fd2abe4d2b5e2bb4275b0cf50d4e8b8b8b8b8b8b8b8b8b8b8b604051808c6001600160a01b031681526020018b6001600160a01b031681526020018a6001600160a01b031681526020018981526020018060200180602001866001600160a01b0316815260200185815260200184815260200183810383528a8a82818152602001925080828437600083820152601f01601f191690910184810383528881526020019050888880828437600083820152604051601f909101601f19169092018290039f50909d5050505050505050505050505050a15050505050505050505050565b6000828152602081815260408083206001600160a01b038516845290915281205460ff16610759576000610798565b604051602001808073455243313832305f4143434550545f4d4147494360601b8152506014019050604051602081830303815290604052805190602001205b9392505050565b836001600160a01b0316639bd9bbc68484846040518463ffffffff1660e01b815260040180846001600160a01b0316815260200183815260200180602001828103825283818151815260200191508051906020019080838360005b838110156108125781810151838201526020016107fa565b50505050905090810190601f16801561083f5780820380516001836020036101000a031916815260200191505b50945050505050600060405180830381600087803b15801561086057600080fd5b505af1158015610874573d6000803e3d6000fd5b5050505050505050565b6040805163fe9d930360e01b815260048101848152602482019283528351604483015283516001600160a01b0387169363fe9d9303938793879390929160640190602085019080838360005b838110156108e25781810151838201526020016108ca565b50505050905090810190601f16801561090f5780820380516001836020036101000a031916815260200191505b509350505050600060405180830381600087803b15801561092f57600080fd5b505af1158015610943573d6000803e3d6000fd5b50505050505050565b6001805460ff1916911515919091179055565b60015460ff161561096f57600080fd5b6000610979610d3d565b90506000816001600160a01b03166370a082318a6040518263ffffffff1660e01b815260040180826001600160a01b0316815260200191505060206040518083038186803b1580156109ca57600080fd5b505afa1580156109de573d6000803e3d6000fd5b505050506040513d60208110156109f457600080fd5b5051604080516370a0823160e01b81526001600160a01b038b811660048301529151929350600092918516916370a0823191602480820192602092909190829003018186803b158015610a4657600080fd5b505afa158015610a5a573d6000803e3d6000fd5b505050506040513d6020811015610a7057600080fd5b810190808051906020019092919050505090507faa3e88aca472e90221daf7d3d601abafb62b120319089d7a2c2f63588da855298b8b8b8b8b8b8b8b8b8b8b604051808c6001600160a01b031681526020018b6001600160a01b031681526020018a6001600160a01b031681526020018981526020018060200180602001866001600160a01b0316815260200185815260200184815260200183810383528a8a82818152602001925080828437600083820152601f01601f191690910184810383528881526020019050888880828437600083820152604051601f909101601f19169092018290039f50909d5050505050505050505050505050a15050505050505050505050565b600154604080516329965a1d60e01b81523060048201527fb281fc8c12954d22544db45de3159a39272895b169a852b314f9cc762e44c53b60248201526001600160a01b038481166044830152915162010000909304909116916329965a1d9160648082019260009290919082900301818387803b158015610bf957600080fd5b505af1158015610c0d573d6000803e3d6000fd5b5050505050565b600180549115156101000261ff0019909216919091179055565b610c587f29ddb589b1fb5fc7cf394961c1adf5f8c6454761adf795e67fe149f658abe89582610d41565b306001600160a01b038216811415610c7357610c7381610cbc565b5050565b610ca17fb281fc8c12954d22544db45de3159a39272895b169a852b314f9cc762e44c53b82610d41565b306001600160a01b038216811415610c7357610c7381610b78565b600154604080516329965a1d60e01b81523060048201527f29ddb589b1fb5fc7cf394961c1adf5f8c6454761adf795e67fe149f658abe89560248201526001600160a01b038481166044830152915162010000909304909116916329965a1d9160648082019260009290919082900301818387803b158015610bf957600080fd5b3390565b6000918252602082815260408084206001600160a01b0390931684529190529020805460ff1916600117905556fea2646970667358221220c676b3f2d390316280c9b8f8aa721840bd863fbb8299487d817dea1ebd1ea6ee64736f6c634300060c0033" . parse () . expect ("invalid bytecode")
    });
    pub struct ERC777SenderRecipientMock<M>(ethers::contract::Contract<M>);
    impl<M> Clone for ERC777SenderRecipientMock<M> {
        fn clone(&self) -> Self {
            ERC777SenderRecipientMock(self.0.clone())
        }
    }
    impl<M> std::ops::Deref for ERC777SenderRecipientMock<M> {
        type Target = ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> std::fmt::Debug for ERC777SenderRecipientMock<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(ERC777SenderRecipientMock))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ethers::providers::Middleware> ERC777SenderRecipientMock<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            ethers::contract::Contract::new(
                address.into(),
                ERC777SENDERRECIPIENTMOCK_ABI.clone(),
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
                ERC777SENDERRECIPIENTMOCK_ABI.clone(),
                ERC777SENDERRECIPIENTMOCK_BYTECODE.clone().into(),
                client,
            );
            let deployer = factory.deploy(constructor_args)?;
            let deployer = ethers::contract::ContractDeployer::new(deployer);
            Ok(deployer)
        }
        #[doc = "Calls the contract's `burn` (0x44d17187) function"]
        pub fn burn(
            &self,
            token: ethers::core::types::Address,
            amount: ethers::core::types::U256,
            data: ethers::core::types::Bytes,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([68, 209, 113, 135], (token, amount, data))
                .expect("method not found (this should never happen)")
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
        #[doc = "Calls the contract's `recipientFor` (0xe0eb2180) function"]
        pub fn recipient_for(
            &self,
            account: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([224, 235, 33, 128], account)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `registerRecipient` (0xa8badaa5) function"]
        pub fn register_recipient(
            &self,
            recipient: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([168, 186, 218, 165], recipient)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `registerSender` (0xe1ecbd30) function"]
        pub fn register_sender(
            &self,
            sender: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([225, 236, 189, 48], sender)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `send` (0x3836ef89) function"]
        pub fn send(
            &self,
            token: ethers::core::types::Address,
            to: ethers::core::types::Address,
            amount: ethers::core::types::U256,
            data: ethers::core::types::Bytes,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([56, 54, 239, 137], (token, to, amount, data))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `senderFor` (0xd2de6474) function"]
        pub fn sender_for(
            &self,
            account: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([210, 222, 100, 116], account)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `setShouldRevertReceive` (0xc97e18fc) function"]
        pub fn set_should_revert_receive(
            &self,
            should_revert: bool,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([201, 126, 24, 252], should_revert)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `setShouldRevertSend` (0x4e4ae5a5) function"]
        pub fn set_should_revert_send(
            &self,
            should_revert: bool,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([78, 74, 229, 165], should_revert)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `tokensReceived` (0x0023de29) function"]
        pub fn tokens_received(
            &self,
            operator: ethers::core::types::Address,
            from: ethers::core::types::Address,
            to: ethers::core::types::Address,
            amount: ethers::core::types::U256,
            user_data: ethers::core::types::Bytes,
            operator_data: ethers::core::types::Bytes,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [0, 35, 222, 41],
                    (operator, from, to, amount, user_data, operator_data),
                )
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `tokensToSend` (0x75ab9782) function"]
        pub fn tokens_to_send(
            &self,
            operator: ethers::core::types::Address,
            from: ethers::core::types::Address,
            to: ethers::core::types::Address,
            amount: ethers::core::types::U256,
            user_data: ethers::core::types::Bytes,
            operator_data: ethers::core::types::Bytes,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [117, 171, 151, 130],
                    (operator, from, to, amount, user_data, operator_data),
                )
                .expect("method not found (this should never happen)")
        }
        #[doc = "Gets the contract's `TokensReceivedCalled` event"]
        pub fn tokens_received_called_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, TokensReceivedCalledFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `TokensToSendCalled` event"]
        pub fn tokens_to_send_called_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, TokensToSendCalledFilter> {
            self.0.event()
        }
        #[doc = r" Returns an [`Event`](#ethers_contract::builders::Event) builder for all events of this contract"]
        pub fn events(
            &self,
        ) -> ethers::contract::builders::Event<M, ERC777SenderRecipientMockEvents> {
            self.0.event_with_filter(Default::default())
        }
    }
    impl<M: ethers::providers::Middleware> From<ethers::contract::Contract<M>>
        for ERC777SenderRecipientMock<M>
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
    #[ethevent(
        name = "TokensReceivedCalled",
        abi = "TokensReceivedCalled(address,address,address,uint256,bytes,bytes,address,uint256,uint256)"
    )]
    pub struct TokensReceivedCalledFilter {
        pub operator: ethers::core::types::Address,
        pub from: ethers::core::types::Address,
        pub to: ethers::core::types::Address,
        pub amount: ethers::core::types::U256,
        pub data: ethers::core::types::Bytes,
        pub operator_data: ethers::core::types::Bytes,
        pub token: ethers::core::types::Address,
        pub from_balance: ethers::core::types::U256,
        pub to_balance: ethers::core::types::U256,
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
        name = "TokensToSendCalled",
        abi = "TokensToSendCalled(address,address,address,uint256,bytes,bytes,address,uint256,uint256)"
    )]
    pub struct TokensToSendCalledFilter {
        pub operator: ethers::core::types::Address,
        pub from: ethers::core::types::Address,
        pub to: ethers::core::types::Address,
        pub amount: ethers::core::types::U256,
        pub data: ethers::core::types::Bytes,
        pub operator_data: ethers::core::types::Bytes,
        pub token: ethers::core::types::Address,
        pub from_balance: ethers::core::types::U256,
        pub to_balance: ethers::core::types::U256,
    }
    #[derive(Debug, Clone, PartialEq, Eq, ethers :: contract :: EthAbiType)]
    pub enum ERC777SenderRecipientMockEvents {
        TokensReceivedCalledFilter(TokensReceivedCalledFilter),
        TokensToSendCalledFilter(TokensToSendCalledFilter),
    }
    impl ethers::contract::EthLogDecode for ERC777SenderRecipientMockEvents {
        fn decode_log(
            log: &ethers::core::abi::RawLog,
        ) -> ::std::result::Result<Self, ethers::core::abi::Error>
        where
            Self: Sized,
        {
            if let Ok(decoded) = TokensReceivedCalledFilter::decode_log(log) {
                return Ok(ERC777SenderRecipientMockEvents::TokensReceivedCalledFilter(
                    decoded,
                ));
            }
            if let Ok(decoded) = TokensToSendCalledFilter::decode_log(log) {
                return Ok(ERC777SenderRecipientMockEvents::TokensToSendCalledFilter(
                    decoded,
                ));
            }
            Err(ethers::core::abi::Error::InvalidData)
        }
    }
    impl ::std::fmt::Display for ERC777SenderRecipientMockEvents {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                ERC777SenderRecipientMockEvents::TokensReceivedCalledFilter(element) => {
                    element.fmt(f)
                }
                ERC777SenderRecipientMockEvents::TokensToSendCalledFilter(element) => {
                    element.fmt(f)
                }
            }
        }
    }
    #[doc = "Container type for all input parameters for the `burn` function with signature `burn(address,uint256,bytes)` and selector `[68, 209, 113, 135]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "burn", abi = "burn(address,uint256,bytes)")]
    pub struct BurnCall {
        pub token: ethers::core::types::Address,
        pub amount: ethers::core::types::U256,
        pub data: ethers::core::types::Bytes,
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
    #[doc = "Container type for all input parameters for the `recipientFor` function with signature `recipientFor(address)` and selector `[224, 235, 33, 128]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "recipientFor", abi = "recipientFor(address)")]
    pub struct RecipientForCall {
        pub account: ethers::core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `registerRecipient` function with signature `registerRecipient(address)` and selector `[168, 186, 218, 165]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "registerRecipient", abi = "registerRecipient(address)")]
    pub struct RegisterRecipientCall {
        pub recipient: ethers::core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `registerSender` function with signature `registerSender(address)` and selector `[225, 236, 189, 48]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "registerSender", abi = "registerSender(address)")]
    pub struct RegisterSenderCall {
        pub sender: ethers::core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `send` function with signature `send(address,address,uint256,bytes)` and selector `[56, 54, 239, 137]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "send", abi = "send(address,address,uint256,bytes)")]
    pub struct SendCall {
        pub token: ethers::core::types::Address,
        pub to: ethers::core::types::Address,
        pub amount: ethers::core::types::U256,
        pub data: ethers::core::types::Bytes,
    }
    #[doc = "Container type for all input parameters for the `senderFor` function with signature `senderFor(address)` and selector `[210, 222, 100, 116]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "senderFor", abi = "senderFor(address)")]
    pub struct SenderForCall {
        pub account: ethers::core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `setShouldRevertReceive` function with signature `setShouldRevertReceive(bool)` and selector `[201, 126, 24, 252]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "setShouldRevertReceive", abi = "setShouldRevertReceive(bool)")]
    pub struct SetShouldRevertReceiveCall {
        pub should_revert: bool,
    }
    #[doc = "Container type for all input parameters for the `setShouldRevertSend` function with signature `setShouldRevertSend(bool)` and selector `[78, 74, 229, 165]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "setShouldRevertSend", abi = "setShouldRevertSend(bool)")]
    pub struct SetShouldRevertSendCall {
        pub should_revert: bool,
    }
    #[doc = "Container type for all input parameters for the `tokensReceived` function with signature `tokensReceived(address,address,address,uint256,bytes,bytes)` and selector `[0, 35, 222, 41]`"]
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
        name = "tokensReceived",
        abi = "tokensReceived(address,address,address,uint256,bytes,bytes)"
    )]
    pub struct TokensReceivedCall {
        pub operator: ethers::core::types::Address,
        pub from: ethers::core::types::Address,
        pub to: ethers::core::types::Address,
        pub amount: ethers::core::types::U256,
        pub user_data: ethers::core::types::Bytes,
        pub operator_data: ethers::core::types::Bytes,
    }
    #[doc = "Container type for all input parameters for the `tokensToSend` function with signature `tokensToSend(address,address,address,uint256,bytes,bytes)` and selector `[117, 171, 151, 130]`"]
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
        name = "tokensToSend",
        abi = "tokensToSend(address,address,address,uint256,bytes,bytes)"
    )]
    pub struct TokensToSendCall {
        pub operator: ethers::core::types::Address,
        pub from: ethers::core::types::Address,
        pub to: ethers::core::types::Address,
        pub amount: ethers::core::types::U256,
        pub user_data: ethers::core::types::Bytes,
        pub operator_data: ethers::core::types::Bytes,
    }
    #[derive(Debug, Clone, PartialEq, Eq, ethers :: contract :: EthAbiType)]
    pub enum ERC777SenderRecipientMockCalls {
        Burn(BurnCall),
        CanImplementInterfaceForAddress(CanImplementInterfaceForAddressCall),
        RecipientFor(RecipientForCall),
        RegisterRecipient(RegisterRecipientCall),
        RegisterSender(RegisterSenderCall),
        Send(SendCall),
        SenderFor(SenderForCall),
        SetShouldRevertReceive(SetShouldRevertReceiveCall),
        SetShouldRevertSend(SetShouldRevertSendCall),
        TokensReceived(TokensReceivedCall),
        TokensToSend(TokensToSendCall),
    }
    impl ethers::core::abi::AbiDecode for ERC777SenderRecipientMockCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::std::result::Result<Self, ethers::core::abi::AbiError> {
            if let Ok(decoded) = <BurnCall as ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(ERC777SenderRecipientMockCalls::Burn(decoded));
            }
            if let Ok(decoded) =
                <CanImplementInterfaceForAddressCall as ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                )
            {
                return Ok(ERC777SenderRecipientMockCalls::CanImplementInterfaceForAddress(decoded));
            }
            if let Ok(decoded) =
                <RecipientForCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777SenderRecipientMockCalls::RecipientFor(decoded));
            }
            if let Ok(decoded) =
                <RegisterRecipientCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777SenderRecipientMockCalls::RegisterRecipient(decoded));
            }
            if let Ok(decoded) =
                <RegisterSenderCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777SenderRecipientMockCalls::RegisterSender(decoded));
            }
            if let Ok(decoded) = <SendCall as ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(ERC777SenderRecipientMockCalls::Send(decoded));
            }
            if let Ok(decoded) =
                <SenderForCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777SenderRecipientMockCalls::SenderFor(decoded));
            }
            if let Ok(decoded) =
                <SetShouldRevertReceiveCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777SenderRecipientMockCalls::SetShouldRevertReceive(
                    decoded,
                ));
            }
            if let Ok(decoded) =
                <SetShouldRevertSendCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777SenderRecipientMockCalls::SetShouldRevertSend(decoded));
            }
            if let Ok(decoded) =
                <TokensReceivedCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777SenderRecipientMockCalls::TokensReceived(decoded));
            }
            if let Ok(decoded) =
                <TokensToSendCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777SenderRecipientMockCalls::TokensToSend(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ethers::core::abi::AbiEncode for ERC777SenderRecipientMockCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                ERC777SenderRecipientMockCalls::Burn(element) => element.encode(),
                ERC777SenderRecipientMockCalls::CanImplementInterfaceForAddress(element) => {
                    element.encode()
                }
                ERC777SenderRecipientMockCalls::RecipientFor(element) => element.encode(),
                ERC777SenderRecipientMockCalls::RegisterRecipient(element) => element.encode(),
                ERC777SenderRecipientMockCalls::RegisterSender(element) => element.encode(),
                ERC777SenderRecipientMockCalls::Send(element) => element.encode(),
                ERC777SenderRecipientMockCalls::SenderFor(element) => element.encode(),
                ERC777SenderRecipientMockCalls::SetShouldRevertReceive(element) => element.encode(),
                ERC777SenderRecipientMockCalls::SetShouldRevertSend(element) => element.encode(),
                ERC777SenderRecipientMockCalls::TokensReceived(element) => element.encode(),
                ERC777SenderRecipientMockCalls::TokensToSend(element) => element.encode(),
            }
        }
    }
    impl ::std::fmt::Display for ERC777SenderRecipientMockCalls {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                ERC777SenderRecipientMockCalls::Burn(element) => element.fmt(f),
                ERC777SenderRecipientMockCalls::CanImplementInterfaceForAddress(element) => {
                    element.fmt(f)
                }
                ERC777SenderRecipientMockCalls::RecipientFor(element) => element.fmt(f),
                ERC777SenderRecipientMockCalls::RegisterRecipient(element) => element.fmt(f),
                ERC777SenderRecipientMockCalls::RegisterSender(element) => element.fmt(f),
                ERC777SenderRecipientMockCalls::Send(element) => element.fmt(f),
                ERC777SenderRecipientMockCalls::SenderFor(element) => element.fmt(f),
                ERC777SenderRecipientMockCalls::SetShouldRevertReceive(element) => element.fmt(f),
                ERC777SenderRecipientMockCalls::SetShouldRevertSend(element) => element.fmt(f),
                ERC777SenderRecipientMockCalls::TokensReceived(element) => element.fmt(f),
                ERC777SenderRecipientMockCalls::TokensToSend(element) => element.fmt(f),
            }
        }
    }
    impl ::std::convert::From<BurnCall> for ERC777SenderRecipientMockCalls {
        fn from(var: BurnCall) -> Self {
            ERC777SenderRecipientMockCalls::Burn(var)
        }
    }
    impl ::std::convert::From<CanImplementInterfaceForAddressCall> for ERC777SenderRecipientMockCalls {
        fn from(var: CanImplementInterfaceForAddressCall) -> Self {
            ERC777SenderRecipientMockCalls::CanImplementInterfaceForAddress(var)
        }
    }
    impl ::std::convert::From<RecipientForCall> for ERC777SenderRecipientMockCalls {
        fn from(var: RecipientForCall) -> Self {
            ERC777SenderRecipientMockCalls::RecipientFor(var)
        }
    }
    impl ::std::convert::From<RegisterRecipientCall> for ERC777SenderRecipientMockCalls {
        fn from(var: RegisterRecipientCall) -> Self {
            ERC777SenderRecipientMockCalls::RegisterRecipient(var)
        }
    }
    impl ::std::convert::From<RegisterSenderCall> for ERC777SenderRecipientMockCalls {
        fn from(var: RegisterSenderCall) -> Self {
            ERC777SenderRecipientMockCalls::RegisterSender(var)
        }
    }
    impl ::std::convert::From<SendCall> for ERC777SenderRecipientMockCalls {
        fn from(var: SendCall) -> Self {
            ERC777SenderRecipientMockCalls::Send(var)
        }
    }
    impl ::std::convert::From<SenderForCall> for ERC777SenderRecipientMockCalls {
        fn from(var: SenderForCall) -> Self {
            ERC777SenderRecipientMockCalls::SenderFor(var)
        }
    }
    impl ::std::convert::From<SetShouldRevertReceiveCall> for ERC777SenderRecipientMockCalls {
        fn from(var: SetShouldRevertReceiveCall) -> Self {
            ERC777SenderRecipientMockCalls::SetShouldRevertReceive(var)
        }
    }
    impl ::std::convert::From<SetShouldRevertSendCall> for ERC777SenderRecipientMockCalls {
        fn from(var: SetShouldRevertSendCall) -> Self {
            ERC777SenderRecipientMockCalls::SetShouldRevertSend(var)
        }
    }
    impl ::std::convert::From<TokensReceivedCall> for ERC777SenderRecipientMockCalls {
        fn from(var: TokensReceivedCall) -> Self {
            ERC777SenderRecipientMockCalls::TokensReceived(var)
        }
    }
    impl ::std::convert::From<TokensToSendCall> for ERC777SenderRecipientMockCalls {
        fn from(var: TokensToSendCall) -> Self {
            ERC777SenderRecipientMockCalls::TokensToSend(var)
        }
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
