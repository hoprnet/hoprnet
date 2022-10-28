pub use erc677_bridge_token::*;
#[allow(clippy::too_many_arguments, non_camel_case_types)]
pub mod erc677_bridge_token {
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
    #[doc = "ERC677BridgeToken was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    # [rustfmt :: skip] const __ABI : & str = "[{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"mintingFinished\",\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"name\",\"outputs\":[{\"name\":\"\",\"type\":\"string\",\"components\":[]}]},{\"inputs\":[{\"name\":\"_spender\",\"type\":\"address\",\"components\":[]},{\"name\":\"_value\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"approve\",\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"name\":\"_bridgeContract\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"setBridgeContract\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"totalSupply\",\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"name\":\"_from\",\"type\":\"address\",\"components\":[]},{\"name\":\"_to\",\"type\":\"address\",\"components\":[]},{\"name\":\"_value\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"transferFrom\",\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"decimals\",\"outputs\":[{\"name\":\"\",\"type\":\"uint8\",\"components\":[]}]},{\"inputs\":[{\"name\":\"spender\",\"type\":\"address\",\"components\":[]},{\"name\":\"addedValue\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"increaseAllowance\",\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"name\":\"_to\",\"type\":\"address\",\"components\":[]},{\"name\":\"_value\",\"type\":\"uint256\",\"components\":[]},{\"name\":\"_data\",\"type\":\"bytes\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"transferAndCall\",\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"name\":\"_to\",\"type\":\"address\",\"components\":[]},{\"name\":\"_amount\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"mint\",\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"name\":\"_value\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"burn\",\"outputs\":[]},{\"inputs\":[{\"name\":\"_spender\",\"type\":\"address\",\"components\":[]},{\"name\":\"_subtractedValue\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"decreaseApproval\",\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"name\":\"_token\",\"type\":\"address\",\"components\":[]},{\"name\":\"_to\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"claimTokens\",\"outputs\":[]},{\"inputs\":[{\"name\":\"_owner\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"balanceOf\",\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"renounceOwnership\",\"outputs\":[]},{\"inputs\":[{\"name\":\"_address\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"isBridge\",\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"finishMinting\",\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"pure\",\"type\":\"function\",\"name\":\"getTokenInterfacesVersion\",\"outputs\":[{\"name\":\"major\",\"type\":\"uint64\",\"components\":[]},{\"name\":\"minor\",\"type\":\"uint64\",\"components\":[]},{\"name\":\"patch\",\"type\":\"uint64\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"owner\",\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"symbol\",\"outputs\":[{\"name\":\"\",\"type\":\"string\",\"components\":[]}]},{\"inputs\":[{\"name\":\"spender\",\"type\":\"address\",\"components\":[]},{\"name\":\"subtractedValue\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"decreaseAllowance\",\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"name\":\"_to\",\"type\":\"address\",\"components\":[]},{\"name\":\"_value\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"transfer\",\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"bridgeContract\",\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[{\"name\":\"_spender\",\"type\":\"address\",\"components\":[]},{\"name\":\"_addedValue\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"increaseApproval\",\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"name\":\"_owner\",\"type\":\"address\",\"components\":[]},{\"name\":\"_spender\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"allowance\",\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"name\":\"_newOwner\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"transferOwnership\",\"outputs\":[]},{\"inputs\":[{\"name\":\"_name\",\"type\":\"string\",\"components\":[]},{\"name\":\"_symbol\",\"type\":\"string\",\"components\":[]},{\"name\":\"_decimals\",\"type\":\"uint8\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"constructor\",\"outputs\":[]},{\"inputs\":[{\"name\":\"to\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"name\":\"amount\",\"type\":\"uint256\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"Mint\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[],\"type\":\"event\",\"name\":\"MintFinished\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"name\":\"previousOwner\",\"type\":\"address\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"OwnershipRenounced\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"name\":\"previousOwner\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"name\":\"newOwner\",\"type\":\"address\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"OwnershipTransferred\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"name\":\"burner\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"name\":\"value\",\"type\":\"uint256\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"Burn\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"name\":\"from\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"name\":\"to\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"name\":\"value\",\"type\":\"uint256\",\"components\":[],\"indexed\":false},{\"name\":\"data\",\"type\":\"bytes\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"Transfer\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"name\":\"owner\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"name\":\"spender\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"name\":\"value\",\"type\":\"uint256\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"Approval\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"name\":\"from\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"name\":\"to\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"name\":\"value\",\"type\":\"uint256\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"Transfer\",\"outputs\":[],\"anonymous\":false}]" ;
    #[doc = r" The parsed JSON-ABI of the contract."]
    pub static ERC677BRIDGETOKEN_ABI: ethers::contract::Lazy<ethers::core::abi::Abi> =
        ethers::contract::Lazy::new(|| {
            ethers::core::utils::__serde_json::from_str(__ABI).expect("invalid abi")
        });
    #[doc = r" Bytecode of the #name contract"]
    pub static ERC677BRIDGETOKEN_BYTECODE: ethers::contract::Lazy<ethers::core::types::Bytes> =
        ethers::contract::Lazy::new(|| {
            "0x60806040526006805460a060020a60ff02191690553480156200002157600080fd5b506040516200159a3803806200159a8339810160409081528151602080840151928401519184018051909493909301928491849184916200006891600091860190620000b2565b5081516200007e906001906020850190620000b2565b506002805460ff90921660ff19909216919091179055505060068054600160a060020a031916331790555062000157915050565b828054600181600116156101000203166002900490600052602060002090601f016020900481019282601f10620000f557805160ff191683800117855562000125565b8280016001018555821562000125579182015b828111156200012557825182559160200191906001019062000108565b506200013392915062000137565b5090565b6200015491905b808211156200013357600081556001016200013e565b90565b61143380620001676000396000f3006080604052600436106101375760003560e01c63ffffffff16806305d2035b1461013c57806306fdde0314610165578063095ea7b3146101ef5780630b26cf661461021357806318160ddd1461023657806323b872dd1461025d578063313ce5671461028757806339509351146102b25780634000aea0146102d657806340c10f191461030757806342966c681461032b578063661884631461034357806369ffa08a1461036757806370a082311461038e578063715018a6146103af578063726600ce146103c45780637d64bcb4146103e5578063859ba28c146103fa5780638da5cb5b1461043b57806395d89b411461046c578063a457c2d714610481578063a9059cbb146104a5578063cd596583146104c9578063d73dd623146104de578063dd62ed3e14610502578063f2fde38b14610529575b600080fd5b34801561014857600080fd5b5061015161054a565b604080519115158252519081900360200190f35b34801561017157600080fd5b5061017a61056b565b6040805160208082528351818301528351919283929083019185019080838360005b838110156101b457818101518382015260200161019c565b50505050905090810190601f1680156101e15780820380516001836020036101000a031916815260200191505b509250505060405180910390f35b3480156101fb57600080fd5b50610151600160a060020a03600435166024356105f9565b34801561021f57600080fd5b50610234600160a060020a036004351661065f565b005b34801561024257600080fd5b5061024b6106b9565b60408051918252519081900360200190f35b34801561026957600080fd5b50610151600160a060020a03600435811690602435166044356106bf565b34801561029357600080fd5b5061029c6106ec565b6040805160ff9092168252519081900360200190f35b3480156102be57600080fd5b50610151600160a060020a03600435166024356106f5565b3480156102e257600080fd5b5061015160048035600160a060020a0316906024803591604435918201910135610708565b34801561031357600080fd5b50610151600160a060020a0360043516602435610819565b34801561033757600080fd5b50610234600435610924565b34801561034f57600080fd5b50610151600160a060020a0360043516602435610931565b34801561037357600080fd5b50610234600160a060020a0360043581169060243516610a20565b34801561039a57600080fd5b5061024b600160a060020a0360043516610a45565b3480156103bb57600080fd5b50610234610a60565b3480156103d057600080fd5b50610151600160a060020a0360043516610a77565b3480156103f157600080fd5b50610151610a8b565b34801561040657600080fd5b5061040f610a92565b6040805167ffffffffffffffff9485168152928416602084015292168183015290519081900360600190f35b34801561044757600080fd5b50610450610a9d565b60408051600160a060020a039092168252519081900360200190f35b34801561047857600080fd5b5061017a610aac565b34801561048d57600080fd5b50610151600160a060020a0360043516602435610b06565b3480156104b157600080fd5b50610151600160a060020a0360043516602435610b12565b3480156104d557600080fd5b50610450610b3d565b3480156104ea57600080fd5b50610151600160a060020a0360043516602435610b4c565b34801561050e57600080fd5b5061024b600160a060020a0360043581169060243516610be5565b34801561053557600080fd5b50610234600160a060020a0360043516610c10565b60065474010000000000000000000000000000000000000000900460ff1681565b6000805460408051602060026001851615610100026000190190941693909304601f810184900484028201840190925281815292918301828280156105f15780601f106105c6576101008083540402835291602001916105f1565b820191906000526020600020905b8154815290600101906020018083116105d457829003601f168201915b505050505081565b336000818152600560209081526040808320600160a060020a038716808552908352818420869055815186815291519394909390927f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925928290030190a350600192915050565b600654600160a060020a0316331461067657600080fd5b61067f81610c30565b151561068a57600080fd5b6007805473ffffffffffffffffffffffffffffffffffffffff1916600160a060020a0392909216919091179055565b60045490565b60006106cc848484610c38565b15156106d757600080fd5b6106e2848484610d9d565b5060019392505050565b60025460ff1681565b60006107018383610b4c565b9392505050565b600084600160a060020a0381161580159061072c5750600160a060020a0381163014155b151561073757600080fd5b6107418686610dd9565b151561074c57600080fd5b85600160a060020a031633600160a060020a03167fe19260aff97b920c7df27010903aeb9c8d2be5d310a2c67824cf3f15396e4c16878787604051808481526020018060200182810382528484828181526020019250808284376040519201829003965090945050505050a36107c186610c30565b1561080d5761080233878787878080601f01602080910402602001604051908101604052809392919081815260200183838082843750610de5945050505050565b151561080d57600080fd5b50600195945050505050565b600654600090600160a060020a0316331461083357600080fd5b60065474010000000000000000000000000000000000000000900460ff161561085b57600080fd5b60045461086e908363ffffffff610f5f16565b600455600160a060020a03831660009081526003602052604090205461089a908363ffffffff610f5f16565b600160a060020a038416600081815260036020908152604091829020939093558051858152905191927f0f6798a560793a54c3bcfe86a93cde1e73087d944c0ea20544137d412139688592918290030190a2604080518381529051600160a060020a038516916000916000805160206113e88339815191529181900360200190a350600192915050565b61092e3382610f72565b50565b336000908152600560209081526040808320600160a060020a038616845290915281205480831061098557336000908152600560209081526040808320600160a060020a03881684529091528120556109ba565b610995818463ffffffff61106116565b336000908152600560209081526040808320600160a060020a03891684529091529020555b336000818152600560209081526040808320600160a060020a0389168085529083529281902054815190815290519293927f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925929181900390910190a35060019392505050565b600654600160a060020a03163314610a3757600080fd5b610a418282611073565b5050565b600160a060020a031660009081526003602052604090205490565b600654600160a060020a0316331461013757600080fd5b600754600160a060020a0390811691161490565b6000806000fd5b600260056000909192565b600654600160a060020a031681565b60018054604080516020600284861615610100026000190190941693909304601f810184900484028201840190925281815292918301828280156105f15780601f106105c6576101008083540402835291602001916105f1565b60006107018383610931565b6000610b1e8383610dd9565b1515610b2957600080fd5b610b34338484610d9d565b50600192915050565b600754600160a060020a031690565b336000908152600560209081526040808320600160a060020a0386168452909152812054610b80908363ffffffff610f5f16565b336000818152600560209081526040808320600160a060020a0389168085529083529281902085905580519485525191937f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925929081900390910190a350600192915050565b600160a060020a03918216600090815260056020908152604080832093909416825291909152205490565b600654600160a060020a03163314610c2757600080fd5b61092e816110b1565b6000903b1190565b600160a060020a038316600090815260036020526040812054821115610c5d57600080fd5b600160a060020a0384166000908152600560209081526040808320338452909152902054821115610c8d57600080fd5b600160a060020a0383161515610ca257600080fd5b600160a060020a038416600090815260036020526040902054610ccb908363ffffffff61106116565b600160a060020a038086166000908152600360205260408082209390935590851681522054610d00908363ffffffff610f5f16565b600160a060020a038085166000908152600360209081526040808320949094559187168152600582528281203382529091522054610d44908363ffffffff61106116565b600160a060020a03808616600081815260056020908152604080832033845282529182902094909455805186815290519287169391926000805160206113e8833981519152929181900390910190a35060019392505050565b610da682610a77565b15610dd457604080516000815260208101909152610dc990849084908490610de5565b1515610dd457600080fd5b505050565b6000610701838361112f565b600083600160a060020a031663a4c0ed3660e01b8685856040516024018084600160a060020a0316600160a060020a0316815260200183815260200180602001828103825283818151815260200191508051906020019080838360005b83811015610e5a578181015183820152602001610e42565b50505050905090810190601f168015610e875780820380516001836020036101000a031916815260200191505b5060408051601f198184030181529181526020820180517bffffffffffffffffffffffffffffffffffffffffffffffffffffffff167fffffffff00000000000000000000000000000000000000000000000000000000909916989098178852518151919790965086955093509150819050838360005b83811015610f15578181015183820152602001610efd565b50505050905090810190601f168015610f425780820380516001836020036101000a031916815260200191505b509150506000604051808303816000865af1979650505050505050565b81810182811015610f6c57fe5b92915050565b600160a060020a038216600090815260036020526040902054811115610f9757600080fd5b600160a060020a038216600090815260036020526040902054610fc0908263ffffffff61106116565b600160a060020a038316600090815260036020526040902055600454610fec908263ffffffff61106116565b600455604080518281529051600160a060020a038416917fcc16f5dbb4873280815c1ee09dbd06736cffcc184412cf7a71a0fdb75d397ca5919081900360200190a2604080518281529051600091600160a060020a038516916000805160206113e88339815191529181900360200190a35050565b60008282111561106d57fe5b50900390565b80600160a060020a038116151561108957600080fd5b600160a060020a03831615156110a7576110a2826111fe565b610dd4565b610dd4838361120a565b600160a060020a03811615156110c657600080fd5b600654604051600160a060020a038084169216907f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e090600090a36006805473ffffffffffffffffffffffffffffffffffffffff1916600160a060020a0392909216919091179055565b3360009081526003602052604081205482111561114b57600080fd5b600160a060020a038316151561116057600080fd5b33600090815260036020526040902054611180908363ffffffff61106116565b3360009081526003602052604080822092909255600160a060020a038516815220546111b2908363ffffffff610f5f16565b600160a060020a0384166000818152600360209081526040918290209390935580518581529051919233926000805160206113e88339815191529281900390910190a350600192915050565b3031610a4182826112bd565b604080517f70a0823100000000000000000000000000000000000000000000000000000000815230600482015290518391600091600160a060020a038416916370a0823191602480830192602092919082900301818787803b15801561126f57600080fd5b505af1158015611283573d6000803e3d6000fd5b505050506040513d602081101561129957600080fd5b505190506112b7600160a060020a038516848363ffffffff61132516565b50505050565b604051600160a060020a0383169082156108fc029083906000818181858888f193505050501515610a415780826112f26113b7565b600160a060020a039091168152604051908190036020019082f08015801561131e573d6000803e3d6000fd5b5050505050565b82600160a060020a031663a9059cbb83836040518363ffffffff1660e01b81526004018083600160a060020a0316600160a060020a0316815260200182815260200192505050600060405180830381600087803b15801561138557600080fd5b505af1158015611399573d6000803e3d6000fd5b505050503d15610dd45760206000803e6000511515610dd457600080fd5b6040516021806113c7833901905600608060405260405160208060218339810160405251600160a060020a038116ff00ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3efa165627a7a72305820af202a595a7985769901bb9cc24203218b161ea232665db01756f795c56dc3a20029" . parse () . expect ("invalid bytecode")
        });
    pub struct ERC677BridgeToken<M>(ethers::contract::Contract<M>);
    impl<M> Clone for ERC677BridgeToken<M> {
        fn clone(&self) -> Self {
            ERC677BridgeToken(self.0.clone())
        }
    }
    impl<M> std::ops::Deref for ERC677BridgeToken<M> {
        type Target = ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> std::fmt::Debug for ERC677BridgeToken<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(ERC677BridgeToken))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ethers::providers::Middleware> ERC677BridgeToken<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            ethers::contract::Contract::new(address.into(), ERC677BRIDGETOKEN_ABI.clone(), client)
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
                ERC677BRIDGETOKEN_ABI.clone(),
                ERC677BRIDGETOKEN_BYTECODE.clone().into(),
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
        #[doc = "Calls the contract's `bridgeContract` (0xcd596583) function"]
        pub fn bridge_contract(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Address> {
            self.0
                .method_hash([205, 89, 101, 131], ())
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
        #[doc = "Calls the contract's `decimals` (0x313ce567) function"]
        pub fn decimals(&self) -> ethers::contract::builders::ContractCall<M, u8> {
            self.0
                .method_hash([49, 60, 229, 103], ())
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
        #[doc = "Calls the contract's `getTokenInterfacesVersion` (0x859ba28c) function"]
        pub fn get_token_interfaces_version(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, (u64, u64, u64)> {
            self.0
                .method_hash([133, 155, 162, 140], ())
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
        #[doc = "Calls the contract's `isBridge` (0x726600ce) function"]
        pub fn is_bridge(
            &self,
            address: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([114, 102, 0, 206], address)
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
        #[doc = "Calls the contract's `name` (0x06fdde03) function"]
        pub fn name(&self) -> ethers::contract::builders::ContractCall<M, String> {
            self.0
                .method_hash([6, 253, 222, 3], ())
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
        #[doc = "Calls the contract's `setBridgeContract` (0x0b26cf66) function"]
        pub fn set_bridge_contract(
            &self,
            bridge_contract: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([11, 38, 207, 102], bridge_contract)
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
            to: ethers::core::types::Address,
            value: ethers::core::types::U256,
            data: ethers::core::types::Bytes,
        ) -> ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([64, 0, 174, 160], (to, value, data))
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
        #[doc = "Gets the contract's `Burn` event"]
        pub fn burn_filter(&self) -> ethers::contract::builders::Event<M, BurnFilter> {
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
        pub fn transfer_1_filter(&self) -> ethers::contract::builders::Event<M, Transfer1Filter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `Transfer` event"]
        pub fn transfer_2_filter(&self) -> ethers::contract::builders::Event<M, Transfer2Filter> {
            self.0.event()
        }
        #[doc = r" Returns an [`Event`](#ethers_contract::builders::Event) builder for all events of this contract"]
        pub fn events(&self) -> ethers::contract::builders::Event<M, ERC677BridgeTokenEvents> {
            self.0.event_with_filter(Default::default())
        }
    }
    impl<M: ethers::providers::Middleware> From<ethers::contract::Contract<M>>
        for ERC677BridgeToken<M>
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
    #[ethevent(name = "Burn", abi = "Burn(address,uint256)")]
    pub struct BurnFilter {
        #[ethevent(indexed)]
        pub burner: ethers::core::types::Address,
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
    pub enum ERC677BridgeTokenEvents {
        ApprovalFilter(ApprovalFilter),
        BurnFilter(BurnFilter),
        MintFilter(MintFilter),
        MintFinishedFilter(MintFinishedFilter),
        OwnershipRenouncedFilter(OwnershipRenouncedFilter),
        OwnershipTransferredFilter(OwnershipTransferredFilter),
        Transfer1Filter(Transfer1Filter),
        Transfer2Filter(Transfer2Filter),
    }
    impl ethers::contract::EthLogDecode for ERC677BridgeTokenEvents {
        fn decode_log(
            log: &ethers::core::abi::RawLog,
        ) -> ::std::result::Result<Self, ethers::core::abi::Error>
        where
            Self: Sized,
        {
            if let Ok(decoded) = ApprovalFilter::decode_log(log) {
                return Ok(ERC677BridgeTokenEvents::ApprovalFilter(decoded));
            }
            if let Ok(decoded) = BurnFilter::decode_log(log) {
                return Ok(ERC677BridgeTokenEvents::BurnFilter(decoded));
            }
            if let Ok(decoded) = MintFilter::decode_log(log) {
                return Ok(ERC677BridgeTokenEvents::MintFilter(decoded));
            }
            if let Ok(decoded) = MintFinishedFilter::decode_log(log) {
                return Ok(ERC677BridgeTokenEvents::MintFinishedFilter(decoded));
            }
            if let Ok(decoded) = OwnershipRenouncedFilter::decode_log(log) {
                return Ok(ERC677BridgeTokenEvents::OwnershipRenouncedFilter(decoded));
            }
            if let Ok(decoded) = OwnershipTransferredFilter::decode_log(log) {
                return Ok(ERC677BridgeTokenEvents::OwnershipTransferredFilter(decoded));
            }
            if let Ok(decoded) = Transfer1Filter::decode_log(log) {
                return Ok(ERC677BridgeTokenEvents::Transfer1Filter(decoded));
            }
            if let Ok(decoded) = Transfer2Filter::decode_log(log) {
                return Ok(ERC677BridgeTokenEvents::Transfer2Filter(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData)
        }
    }
    impl ::std::fmt::Display for ERC677BridgeTokenEvents {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                ERC677BridgeTokenEvents::ApprovalFilter(element) => element.fmt(f),
                ERC677BridgeTokenEvents::BurnFilter(element) => element.fmt(f),
                ERC677BridgeTokenEvents::MintFilter(element) => element.fmt(f),
                ERC677BridgeTokenEvents::MintFinishedFilter(element) => element.fmt(f),
                ERC677BridgeTokenEvents::OwnershipRenouncedFilter(element) => element.fmt(f),
                ERC677BridgeTokenEvents::OwnershipTransferredFilter(element) => element.fmt(f),
                ERC677BridgeTokenEvents::Transfer1Filter(element) => element.fmt(f),
                ERC677BridgeTokenEvents::Transfer2Filter(element) => element.fmt(f),
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
    #[doc = "Container type for all input parameters for the `bridgeContract` function with signature `bridgeContract()` and selector `[205, 89, 101, 131]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "bridgeContract", abi = "bridgeContract()")]
    pub struct BridgeContractCall;
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
    #[doc = "Container type for all input parameters for the `getTokenInterfacesVersion` function with signature `getTokenInterfacesVersion()` and selector `[133, 155, 162, 140]`"]
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
        name = "getTokenInterfacesVersion",
        abi = "getTokenInterfacesVersion()"
    )]
    pub struct GetTokenInterfacesVersionCall;
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
    #[doc = "Container type for all input parameters for the `isBridge` function with signature `isBridge(address)` and selector `[114, 102, 0, 206]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "isBridge", abi = "isBridge(address)")]
    pub struct IsBridgeCall {
        pub address: ethers::core::types::Address,
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
    #[doc = "Container type for all input parameters for the `setBridgeContract` function with signature `setBridgeContract(address)` and selector `[11, 38, 207, 102]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "setBridgeContract", abi = "setBridgeContract(address)")]
    pub struct SetBridgeContractCall {
        pub bridge_contract: ethers::core::types::Address,
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
    pub struct TransferAndCallCall {
        pub to: ethers::core::types::Address,
        pub value: ethers::core::types::U256,
        pub data: ethers::core::types::Bytes,
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
    pub enum ERC677BridgeTokenCalls {
        Allowance(AllowanceCall),
        Approve(ApproveCall),
        BalanceOf(BalanceOfCall),
        BridgeContract(BridgeContractCall),
        Burn(BurnCall),
        ClaimTokens(ClaimTokensCall),
        Decimals(DecimalsCall),
        DecreaseAllowance(DecreaseAllowanceCall),
        DecreaseApproval(DecreaseApprovalCall),
        FinishMinting(FinishMintingCall),
        GetTokenInterfacesVersion(GetTokenInterfacesVersionCall),
        IncreaseAllowance(IncreaseAllowanceCall),
        IncreaseApproval(IncreaseApprovalCall),
        IsBridge(IsBridgeCall),
        Mint(MintCall),
        MintingFinished(MintingFinishedCall),
        Name(NameCall),
        Owner(OwnerCall),
        RenounceOwnership(RenounceOwnershipCall),
        SetBridgeContract(SetBridgeContractCall),
        Symbol(SymbolCall),
        TotalSupply(TotalSupplyCall),
        Transfer(TransferCall),
        TransferAndCall(TransferAndCallCall),
        TransferFrom(TransferFromCall),
        TransferOwnership(TransferOwnershipCall),
    }
    impl ethers::core::abi::AbiDecode for ERC677BridgeTokenCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::std::result::Result<Self, ethers::core::abi::AbiError> {
            if let Ok(decoded) =
                <AllowanceCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC677BridgeTokenCalls::Allowance(decoded));
            }
            if let Ok(decoded) =
                <ApproveCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC677BridgeTokenCalls::Approve(decoded));
            }
            if let Ok(decoded) =
                <BalanceOfCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC677BridgeTokenCalls::BalanceOf(decoded));
            }
            if let Ok(decoded) =
                <BridgeContractCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC677BridgeTokenCalls::BridgeContract(decoded));
            }
            if let Ok(decoded) = <BurnCall as ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(ERC677BridgeTokenCalls::Burn(decoded));
            }
            if let Ok(decoded) =
                <ClaimTokensCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC677BridgeTokenCalls::ClaimTokens(decoded));
            }
            if let Ok(decoded) =
                <DecimalsCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC677BridgeTokenCalls::Decimals(decoded));
            }
            if let Ok(decoded) =
                <DecreaseAllowanceCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC677BridgeTokenCalls::DecreaseAllowance(decoded));
            }
            if let Ok(decoded) =
                <DecreaseApprovalCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC677BridgeTokenCalls::DecreaseApproval(decoded));
            }
            if let Ok(decoded) =
                <FinishMintingCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC677BridgeTokenCalls::FinishMinting(decoded));
            }
            if let Ok(decoded) =
                <GetTokenInterfacesVersionCall as ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                )
            {
                return Ok(ERC677BridgeTokenCalls::GetTokenInterfacesVersion(decoded));
            }
            if let Ok(decoded) =
                <IncreaseAllowanceCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC677BridgeTokenCalls::IncreaseAllowance(decoded));
            }
            if let Ok(decoded) =
                <IncreaseApprovalCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC677BridgeTokenCalls::IncreaseApproval(decoded));
            }
            if let Ok(decoded) =
                <IsBridgeCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC677BridgeTokenCalls::IsBridge(decoded));
            }
            if let Ok(decoded) = <MintCall as ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(ERC677BridgeTokenCalls::Mint(decoded));
            }
            if let Ok(decoded) =
                <MintingFinishedCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC677BridgeTokenCalls::MintingFinished(decoded));
            }
            if let Ok(decoded) = <NameCall as ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(ERC677BridgeTokenCalls::Name(decoded));
            }
            if let Ok(decoded) = <OwnerCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC677BridgeTokenCalls::Owner(decoded));
            }
            if let Ok(decoded) =
                <RenounceOwnershipCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC677BridgeTokenCalls::RenounceOwnership(decoded));
            }
            if let Ok(decoded) =
                <SetBridgeContractCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC677BridgeTokenCalls::SetBridgeContract(decoded));
            }
            if let Ok(decoded) = <SymbolCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC677BridgeTokenCalls::Symbol(decoded));
            }
            if let Ok(decoded) =
                <TotalSupplyCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC677BridgeTokenCalls::TotalSupply(decoded));
            }
            if let Ok(decoded) =
                <TransferCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC677BridgeTokenCalls::Transfer(decoded));
            }
            if let Ok(decoded) =
                <TransferAndCallCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC677BridgeTokenCalls::TransferAndCall(decoded));
            }
            if let Ok(decoded) =
                <TransferFromCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC677BridgeTokenCalls::TransferFrom(decoded));
            }
            if let Ok(decoded) =
                <TransferOwnershipCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC677BridgeTokenCalls::TransferOwnership(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ethers::core::abi::AbiEncode for ERC677BridgeTokenCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                ERC677BridgeTokenCalls::Allowance(element) => element.encode(),
                ERC677BridgeTokenCalls::Approve(element) => element.encode(),
                ERC677BridgeTokenCalls::BalanceOf(element) => element.encode(),
                ERC677BridgeTokenCalls::BridgeContract(element) => element.encode(),
                ERC677BridgeTokenCalls::Burn(element) => element.encode(),
                ERC677BridgeTokenCalls::ClaimTokens(element) => element.encode(),
                ERC677BridgeTokenCalls::Decimals(element) => element.encode(),
                ERC677BridgeTokenCalls::DecreaseAllowance(element) => element.encode(),
                ERC677BridgeTokenCalls::DecreaseApproval(element) => element.encode(),
                ERC677BridgeTokenCalls::FinishMinting(element) => element.encode(),
                ERC677BridgeTokenCalls::GetTokenInterfacesVersion(element) => element.encode(),
                ERC677BridgeTokenCalls::IncreaseAllowance(element) => element.encode(),
                ERC677BridgeTokenCalls::IncreaseApproval(element) => element.encode(),
                ERC677BridgeTokenCalls::IsBridge(element) => element.encode(),
                ERC677BridgeTokenCalls::Mint(element) => element.encode(),
                ERC677BridgeTokenCalls::MintingFinished(element) => element.encode(),
                ERC677BridgeTokenCalls::Name(element) => element.encode(),
                ERC677BridgeTokenCalls::Owner(element) => element.encode(),
                ERC677BridgeTokenCalls::RenounceOwnership(element) => element.encode(),
                ERC677BridgeTokenCalls::SetBridgeContract(element) => element.encode(),
                ERC677BridgeTokenCalls::Symbol(element) => element.encode(),
                ERC677BridgeTokenCalls::TotalSupply(element) => element.encode(),
                ERC677BridgeTokenCalls::Transfer(element) => element.encode(),
                ERC677BridgeTokenCalls::TransferAndCall(element) => element.encode(),
                ERC677BridgeTokenCalls::TransferFrom(element) => element.encode(),
                ERC677BridgeTokenCalls::TransferOwnership(element) => element.encode(),
            }
        }
    }
    impl ::std::fmt::Display for ERC677BridgeTokenCalls {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                ERC677BridgeTokenCalls::Allowance(element) => element.fmt(f),
                ERC677BridgeTokenCalls::Approve(element) => element.fmt(f),
                ERC677BridgeTokenCalls::BalanceOf(element) => element.fmt(f),
                ERC677BridgeTokenCalls::BridgeContract(element) => element.fmt(f),
                ERC677BridgeTokenCalls::Burn(element) => element.fmt(f),
                ERC677BridgeTokenCalls::ClaimTokens(element) => element.fmt(f),
                ERC677BridgeTokenCalls::Decimals(element) => element.fmt(f),
                ERC677BridgeTokenCalls::DecreaseAllowance(element) => element.fmt(f),
                ERC677BridgeTokenCalls::DecreaseApproval(element) => element.fmt(f),
                ERC677BridgeTokenCalls::FinishMinting(element) => element.fmt(f),
                ERC677BridgeTokenCalls::GetTokenInterfacesVersion(element) => element.fmt(f),
                ERC677BridgeTokenCalls::IncreaseAllowance(element) => element.fmt(f),
                ERC677BridgeTokenCalls::IncreaseApproval(element) => element.fmt(f),
                ERC677BridgeTokenCalls::IsBridge(element) => element.fmt(f),
                ERC677BridgeTokenCalls::Mint(element) => element.fmt(f),
                ERC677BridgeTokenCalls::MintingFinished(element) => element.fmt(f),
                ERC677BridgeTokenCalls::Name(element) => element.fmt(f),
                ERC677BridgeTokenCalls::Owner(element) => element.fmt(f),
                ERC677BridgeTokenCalls::RenounceOwnership(element) => element.fmt(f),
                ERC677BridgeTokenCalls::SetBridgeContract(element) => element.fmt(f),
                ERC677BridgeTokenCalls::Symbol(element) => element.fmt(f),
                ERC677BridgeTokenCalls::TotalSupply(element) => element.fmt(f),
                ERC677BridgeTokenCalls::Transfer(element) => element.fmt(f),
                ERC677BridgeTokenCalls::TransferAndCall(element) => element.fmt(f),
                ERC677BridgeTokenCalls::TransferFrom(element) => element.fmt(f),
                ERC677BridgeTokenCalls::TransferOwnership(element) => element.fmt(f),
            }
        }
    }
    impl ::std::convert::From<AllowanceCall> for ERC677BridgeTokenCalls {
        fn from(var: AllowanceCall) -> Self {
            ERC677BridgeTokenCalls::Allowance(var)
        }
    }
    impl ::std::convert::From<ApproveCall> for ERC677BridgeTokenCalls {
        fn from(var: ApproveCall) -> Self {
            ERC677BridgeTokenCalls::Approve(var)
        }
    }
    impl ::std::convert::From<BalanceOfCall> for ERC677BridgeTokenCalls {
        fn from(var: BalanceOfCall) -> Self {
            ERC677BridgeTokenCalls::BalanceOf(var)
        }
    }
    impl ::std::convert::From<BridgeContractCall> for ERC677BridgeTokenCalls {
        fn from(var: BridgeContractCall) -> Self {
            ERC677BridgeTokenCalls::BridgeContract(var)
        }
    }
    impl ::std::convert::From<BurnCall> for ERC677BridgeTokenCalls {
        fn from(var: BurnCall) -> Self {
            ERC677BridgeTokenCalls::Burn(var)
        }
    }
    impl ::std::convert::From<ClaimTokensCall> for ERC677BridgeTokenCalls {
        fn from(var: ClaimTokensCall) -> Self {
            ERC677BridgeTokenCalls::ClaimTokens(var)
        }
    }
    impl ::std::convert::From<DecimalsCall> for ERC677BridgeTokenCalls {
        fn from(var: DecimalsCall) -> Self {
            ERC677BridgeTokenCalls::Decimals(var)
        }
    }
    impl ::std::convert::From<DecreaseAllowanceCall> for ERC677BridgeTokenCalls {
        fn from(var: DecreaseAllowanceCall) -> Self {
            ERC677BridgeTokenCalls::DecreaseAllowance(var)
        }
    }
    impl ::std::convert::From<DecreaseApprovalCall> for ERC677BridgeTokenCalls {
        fn from(var: DecreaseApprovalCall) -> Self {
            ERC677BridgeTokenCalls::DecreaseApproval(var)
        }
    }
    impl ::std::convert::From<FinishMintingCall> for ERC677BridgeTokenCalls {
        fn from(var: FinishMintingCall) -> Self {
            ERC677BridgeTokenCalls::FinishMinting(var)
        }
    }
    impl ::std::convert::From<GetTokenInterfacesVersionCall> for ERC677BridgeTokenCalls {
        fn from(var: GetTokenInterfacesVersionCall) -> Self {
            ERC677BridgeTokenCalls::GetTokenInterfacesVersion(var)
        }
    }
    impl ::std::convert::From<IncreaseAllowanceCall> for ERC677BridgeTokenCalls {
        fn from(var: IncreaseAllowanceCall) -> Self {
            ERC677BridgeTokenCalls::IncreaseAllowance(var)
        }
    }
    impl ::std::convert::From<IncreaseApprovalCall> for ERC677BridgeTokenCalls {
        fn from(var: IncreaseApprovalCall) -> Self {
            ERC677BridgeTokenCalls::IncreaseApproval(var)
        }
    }
    impl ::std::convert::From<IsBridgeCall> for ERC677BridgeTokenCalls {
        fn from(var: IsBridgeCall) -> Self {
            ERC677BridgeTokenCalls::IsBridge(var)
        }
    }
    impl ::std::convert::From<MintCall> for ERC677BridgeTokenCalls {
        fn from(var: MintCall) -> Self {
            ERC677BridgeTokenCalls::Mint(var)
        }
    }
    impl ::std::convert::From<MintingFinishedCall> for ERC677BridgeTokenCalls {
        fn from(var: MintingFinishedCall) -> Self {
            ERC677BridgeTokenCalls::MintingFinished(var)
        }
    }
    impl ::std::convert::From<NameCall> for ERC677BridgeTokenCalls {
        fn from(var: NameCall) -> Self {
            ERC677BridgeTokenCalls::Name(var)
        }
    }
    impl ::std::convert::From<OwnerCall> for ERC677BridgeTokenCalls {
        fn from(var: OwnerCall) -> Self {
            ERC677BridgeTokenCalls::Owner(var)
        }
    }
    impl ::std::convert::From<RenounceOwnershipCall> for ERC677BridgeTokenCalls {
        fn from(var: RenounceOwnershipCall) -> Self {
            ERC677BridgeTokenCalls::RenounceOwnership(var)
        }
    }
    impl ::std::convert::From<SetBridgeContractCall> for ERC677BridgeTokenCalls {
        fn from(var: SetBridgeContractCall) -> Self {
            ERC677BridgeTokenCalls::SetBridgeContract(var)
        }
    }
    impl ::std::convert::From<SymbolCall> for ERC677BridgeTokenCalls {
        fn from(var: SymbolCall) -> Self {
            ERC677BridgeTokenCalls::Symbol(var)
        }
    }
    impl ::std::convert::From<TotalSupplyCall> for ERC677BridgeTokenCalls {
        fn from(var: TotalSupplyCall) -> Self {
            ERC677BridgeTokenCalls::TotalSupply(var)
        }
    }
    impl ::std::convert::From<TransferCall> for ERC677BridgeTokenCalls {
        fn from(var: TransferCall) -> Self {
            ERC677BridgeTokenCalls::Transfer(var)
        }
    }
    impl ::std::convert::From<TransferAndCallCall> for ERC677BridgeTokenCalls {
        fn from(var: TransferAndCallCall) -> Self {
            ERC677BridgeTokenCalls::TransferAndCall(var)
        }
    }
    impl ::std::convert::From<TransferFromCall> for ERC677BridgeTokenCalls {
        fn from(var: TransferFromCall) -> Self {
            ERC677BridgeTokenCalls::TransferFrom(var)
        }
    }
    impl ::std::convert::From<TransferOwnershipCall> for ERC677BridgeTokenCalls {
        fn from(var: TransferOwnershipCall) -> Self {
            ERC677BridgeTokenCalls::TransferOwnership(var)
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
    #[doc = "Container type for all return fields from the `bridgeContract` function with signature `bridgeContract()` and selector `[205, 89, 101, 131]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct BridgeContractReturn(pub ethers::core::types::Address);
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
    #[doc = "Container type for all return fields from the `getTokenInterfacesVersion` function with signature `getTokenInterfacesVersion()` and selector `[133, 155, 162, 140]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct GetTokenInterfacesVersionReturn {
        pub major: u64,
        pub minor: u64,
        pub patch: u64,
    }
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
    #[doc = "Container type for all return fields from the `isBridge` function with signature `isBridge(address)` and selector `[114, 102, 0, 206]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct IsBridgeReturn(pub bool);
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
