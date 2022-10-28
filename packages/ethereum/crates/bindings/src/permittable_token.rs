pub use permittable_token::*;
#[allow(clippy::too_many_arguments, non_camel_case_types)]
pub mod permittable_token {
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
    #[doc = "PermittableToken was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    # [rustfmt :: skip] const __ABI : & str = "[{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"mintingFinished\",\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"name\",\"outputs\":[{\"name\":\"\",\"type\":\"string\",\"components\":[]}]},{\"inputs\":[{\"name\":\"_to\",\"type\":\"address\",\"components\":[]},{\"name\":\"_value\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"approve\",\"outputs\":[{\"name\":\"result\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"name\":\"_bridgeContract\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"setBridgeContract\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"totalSupply\",\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"name\":\"_sender\",\"type\":\"address\",\"components\":[]},{\"name\":\"_recipient\",\"type\":\"address\",\"components\":[]},{\"name\":\"_amount\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"transferFrom\",\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"PERMIT_TYPEHASH\",\"outputs\":[{\"name\":\"\",\"type\":\"bytes32\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"decimals\",\"outputs\":[{\"name\":\"\",\"type\":\"uint8\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"DOMAIN_SEPARATOR\",\"outputs\":[{\"name\":\"\",\"type\":\"bytes32\",\"components\":[]}]},{\"inputs\":[{\"name\":\"_to\",\"type\":\"address\",\"components\":[]},{\"name\":\"_addedValue\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"increaseAllowance\",\"outputs\":[{\"name\":\"result\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"name\":\"_to\",\"type\":\"address\",\"components\":[]},{\"name\":\"_value\",\"type\":\"uint256\",\"components\":[]},{\"name\":\"_data\",\"type\":\"bytes\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"transferAndCall\",\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"name\":\"_to\",\"type\":\"address\",\"components\":[]},{\"name\":\"_amount\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"mint\",\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"name\":\"_value\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"burn\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"version\",\"outputs\":[{\"name\":\"\",\"type\":\"string\",\"components\":[]}]},{\"inputs\":[{\"name\":\"_spender\",\"type\":\"address\",\"components\":[]},{\"name\":\"_subtractedValue\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"decreaseApproval\",\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"name\":\"_token\",\"type\":\"address\",\"components\":[]},{\"name\":\"_to\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"claimTokens\",\"outputs\":[]},{\"inputs\":[{\"name\":\"_owner\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"balanceOf\",\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"renounceOwnership\",\"outputs\":[]},{\"inputs\":[{\"name\":\"_address\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"isBridge\",\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"finishMinting\",\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"name\":\"\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"nonces\",\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"pure\",\"type\":\"function\",\"name\":\"getTokenInterfacesVersion\",\"outputs\":[{\"name\":\"major\",\"type\":\"uint64\",\"components\":[]},{\"name\":\"minor\",\"type\":\"uint64\",\"components\":[]},{\"name\":\"patch\",\"type\":\"uint64\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"owner\",\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[{\"name\":\"_holder\",\"type\":\"address\",\"components\":[]},{\"name\":\"_spender\",\"type\":\"address\",\"components\":[]},{\"name\":\"_nonce\",\"type\":\"uint256\",\"components\":[]},{\"name\":\"_expiry\",\"type\":\"uint256\",\"components\":[]},{\"name\":\"_allowed\",\"type\":\"bool\",\"components\":[]},{\"name\":\"_v\",\"type\":\"uint8\",\"components\":[]},{\"name\":\"_r\",\"type\":\"bytes32\",\"components\":[]},{\"name\":\"_s\",\"type\":\"bytes32\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"permit\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"symbol\",\"outputs\":[{\"name\":\"\",\"type\":\"string\",\"components\":[]}]},{\"inputs\":[{\"name\":\"spender\",\"type\":\"address\",\"components\":[]},{\"name\":\"subtractedValue\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"decreaseAllowance\",\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"name\":\"_to\",\"type\":\"address\",\"components\":[]},{\"name\":\"_value\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"transfer\",\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"name\":\"_to\",\"type\":\"address\",\"components\":[]},{\"name\":\"_amount\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"push\",\"outputs\":[]},{\"inputs\":[{\"name\":\"_from\",\"type\":\"address\",\"components\":[]},{\"name\":\"_to\",\"type\":\"address\",\"components\":[]},{\"name\":\"_amount\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"move\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"PERMIT_TYPEHASH_LEGACY\",\"outputs\":[{\"name\":\"\",\"type\":\"bytes32\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"bridgeContract\",\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[{\"name\":\"_holder\",\"type\":\"address\",\"components\":[]},{\"name\":\"_spender\",\"type\":\"address\",\"components\":[]},{\"name\":\"_value\",\"type\":\"uint256\",\"components\":[]},{\"name\":\"_deadline\",\"type\":\"uint256\",\"components\":[]},{\"name\":\"_v\",\"type\":\"uint8\",\"components\":[]},{\"name\":\"_r\",\"type\":\"bytes32\",\"components\":[]},{\"name\":\"_s\",\"type\":\"bytes32\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"permit\",\"outputs\":[]},{\"inputs\":[{\"name\":\"_spender\",\"type\":\"address\",\"components\":[]},{\"name\":\"_addedValue\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"increaseApproval\",\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"name\":\"_owner\",\"type\":\"address\",\"components\":[]},{\"name\":\"_spender\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"allowance\",\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"name\":\"_from\",\"type\":\"address\",\"components\":[]},{\"name\":\"_amount\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"pull\",\"outputs\":[]},{\"inputs\":[{\"name\":\"_newOwner\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"transferOwnership\",\"outputs\":[]},{\"inputs\":[{\"name\":\"\",\"type\":\"address\",\"components\":[]},{\"name\":\"\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"expirations\",\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"name\":\"_name\",\"type\":\"string\",\"components\":[]},{\"name\":\"_symbol\",\"type\":\"string\",\"components\":[]},{\"name\":\"_decimals\",\"type\":\"uint8\",\"components\":[]},{\"name\":\"_chainId\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"constructor\",\"outputs\":[]},{\"inputs\":[{\"name\":\"to\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"name\":\"amount\",\"type\":\"uint256\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"Mint\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[],\"type\":\"event\",\"name\":\"MintFinished\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"name\":\"previousOwner\",\"type\":\"address\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"OwnershipRenounced\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"name\":\"previousOwner\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"name\":\"newOwner\",\"type\":\"address\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"OwnershipTransferred\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"name\":\"burner\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"name\":\"value\",\"type\":\"uint256\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"Burn\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"name\":\"from\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"name\":\"to\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"name\":\"value\",\"type\":\"uint256\",\"components\":[],\"indexed\":false},{\"name\":\"data\",\"type\":\"bytes\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"Transfer\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"name\":\"owner\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"name\":\"spender\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"name\":\"value\",\"type\":\"uint256\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"Approval\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"name\":\"from\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"name\":\"to\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"name\":\"value\",\"type\":\"uint256\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"Transfer\",\"outputs\":[],\"anonymous\":false}]" ;
    #[doc = r" The parsed JSON-ABI of the contract."]
    pub static PERMITTABLETOKEN_ABI: ethers::contract::Lazy<ethers::core::abi::Abi> =
        ethers::contract::Lazy::new(|| {
            ethers::core::utils::__serde_json::from_str(__ABI).expect("invalid abi")
        });
    #[doc = r" Bytecode of the #name contract"]
    pub static PERMITTABLETOKEN_BYTECODE: ethers::contract::Lazy<ethers::core::types::Bytes> =
        ethers::contract::Lazy::new(|| {
            "0x60806040526006805460a060020a60ff02191690553480156200002157600080fd5b506040516200215d3803806200215d83398101604090815281516020808401519284015160608501519285018051909594909401939092918591859185918491849184916200007691600091860190620002d0565b5081516200008c906001906020850190620002d0565b506002805460ff90921660ff19909216919091179055505060068054600160a060020a03191633179055505050801515620000c657600080fd5b60405180807f454950373132446f6d61696e28737472696e67206e616d652c737472696e672081526020017f76657273696f6e2c75696e7432353620636861696e49642c616464726573732081526020017f766572696679696e67436f6e747261637429000000000000000000000000000081525060520190506040518091039020846040518082805190602001908083835b602083106200017a5780518252601f19909201916020918201910162000159565b51815160209384036101000a600019018019909216911617905260408051929094018290038220828501855260018084527f3100000000000000000000000000000000000000000000000000000000000000928401928352945190965091945090928392508083835b60208310620002045780518252601f199092019160209182019101620001e3565b51815160209384036101000a6000190180199092169116179052604080519290940182900382208282019890985281840196909652606081019690965250608085018690523060a0808701919091528151808703909101815260c09095019081905284519093849350850191508083835b60208310620002965780518252601f19909201916020918201910162000275565b5181516020939093036101000a60001901801990911692169190911790526040519201829003909120600855506200037595505050505050565b828054600181600116156101000203166002900490600052602060002090601f016020900481019282601f106200031357805160ff191683800117855562000343565b8280016001018555821562000343579182015b828111156200034357825182559160200191906001019062000326565b506200035192915062000355565b5090565b6200037291905b808211156200035157600081556001016200035c565b90565b611dd880620003856000396000f3006080604052600436106101b05760003560e01c63ffffffff16806305d2035b146101b557806306fdde03146101de578063095ea7b3146102685780630b26cf661461028c57806318160ddd146102af57806323b872dd146102d657806330adf81f14610300578063313ce567146103155780633644e5151461034057806339509351146103555780634000aea01461037957806340c10f19146103aa57806342966c68146103ce57806354fd4d50146103e657806366188463146103fb57806369ffa08a1461041f57806370a0823114610446578063715018a614610467578063726600ce1461047c5780637d64bcb41461049d5780637ecebe00146104b2578063859ba28c146104d35780638da5cb5b146105145780638fcbaf0c1461054557806395d89b4114610583578063a457c2d714610598578063a9059cbb146105bc578063b753a98c146105e0578063bb35783b14610604578063c6a1dedf1461062e578063cd59658314610643578063d505accf14610658578063d73dd62314610691578063dd62ed3e146106b5578063f2d5d56b146106dc578063f2fde38b14610700578063ff9e884d14610721575b600080fd5b3480156101c157600080fd5b506101ca610748565b604080519115158252519081900360200190f35b3480156101ea57600080fd5b506101f3610769565b6040805160208082528351818301528351919283929083019185019080838360005b8381101561022d578181015183820152602001610215565b50505050905090810190601f16801561025a5780820380516001836020036101000a031916815260200191505b509250505060405180910390f35b34801561027457600080fd5b506101ca600160a060020a03600435166024356107f7565b34801561029857600080fd5b506102ad600160a060020a036004351661080d565b005b3480156102bb57600080fd5b506102c4610867565b60408051918252519081900360200190f35b3480156102e257600080fd5b506101ca600160a060020a036004358116906024351660443561086d565b34801561030c57600080fd5b506102c4610a35565b34801561032157600080fd5b5061032a610a59565b6040805160ff9092168252519081900360200190f35b34801561034c57600080fd5b506102c4610a62565b34801561036157600080fd5b506101ca600160a060020a0360043516602435610a68565b34801561038557600080fd5b506101ca60048035600160a060020a0316906024803591604435918201910135610aa9565b3480156103b657600080fd5b506101ca600160a060020a0360043516602435610bba565b3480156103da57600080fd5b506102ad600435610cc5565b3480156103f257600080fd5b506101f3610cd2565b34801561040757600080fd5b506101ca600160a060020a0360043516602435610d09565b34801561042b57600080fd5b506102ad600160a060020a0360043581169060243516610de6565b34801561045257600080fd5b506102c4600160a060020a0360043516610e0b565b34801561047357600080fd5b506102ad610e26565b34801561048857600080fd5b506101ca600160a060020a0360043516610e3d565b3480156104a957600080fd5b506101ca610e51565b3480156104be57600080fd5b506102c4600160a060020a0360043516610e58565b3480156104df57600080fd5b506104e8610e6a565b6040805167ffffffffffffffff9485168152928416602084015292168183015290519081900360600190f35b34801561052057600080fd5b50610529610e75565b60408051600160a060020a039092168252519081900360200190f35b34801561055157600080fd5b506102ad600160a060020a0360043581169060243516604435606435608435151560ff60a4351660c43560e435610e84565b34801561058f57600080fd5b506101f3610fc2565b3480156105a457600080fd5b506101ca600160a060020a036004351660243561101c565b3480156105c857600080fd5b506101ca600160a060020a036004351660243561102f565b3480156105ec57600080fd5b506102ad600160a060020a0360043516602435611051565b34801561061057600080fd5b506102ad600160a060020a0360043581169060243516604435611061565b34801561063a57600080fd5b506102c4611072565b34801561064f57600080fd5b50610529611096565b34801561066457600080fd5b506102ad600160a060020a036004358116906024351660443560643560ff6084351660a43560c4356110a5565b34801561069d57600080fd5b506101ca600160a060020a0360043516602435611181565b3480156106c157600080fd5b506102c4600160a060020a0360043581169060243516611208565b3480156106e857600080fd5b506102ad600160a060020a0360043516602435611233565b34801561070c57600080fd5b506102ad600160a060020a036004351661123e565b34801561072d57600080fd5b506102c4600160a060020a036004358116906024351661125e565b60065474010000000000000000000000000000000000000000900460ff1681565b6000805460408051602060026001851615610100026000190190941693909304601f810184900484028201840190925281815292918301828280156107ef5780601f106107c4576101008083540402835291602001916107ef565b820191906000526020600020905b8154815290600101906020018083116107d257829003601f168201915b505050505081565b600061080433848461127b565b50600192915050565b600654600160a060020a0316331461082457600080fd5b61082d816112bd565b151561083857600080fd5b6007805473ffffffffffffffffffffffffffffffffffffffff1916600160a060020a0392909216919091179055565b60045490565b600080600160a060020a038516151561088557600080fd5b600160a060020a038416151561089a57600080fd5b600160a060020a0385166000908152600360205260409020546108c3908463ffffffff6112c516565b600160a060020a0380871660009081526003602052604080822093909355908616815220546108f8908463ffffffff6112d716565b600160a060020a038086166000818152600360209081526040918290209490945580518781529051919392891692600080516020611d6d83398151915292918290030190a3600160a060020a0385163314610a1f576109578533611208565b905060001981146109c157610972818463ffffffff6112c516565b600160a060020a038616600081815260056020908152604080832033808552908352928190208590558051948552519193600080516020611d8d833981519152929081900390910190a3610a1f565b600160a060020a0385166000908152600a602090815260408083203384529091529020541580610a145750600160a060020a0385166000908152600a602090815260408083203384529091529020544211155b1515610a1f57600080fd5b610a2a8585856112ea565b506001949350505050565b7f6e71edae12b1b97f4d1f60370fef10105fa2faae0126114a169c64845d6126c981565b60025460ff1681565b60085481565b336000818152600560209081526040808320600160a060020a03871684529091528120549091610804918590610aa4908663ffffffff6112d716565b61127b565b600084600160a060020a03811615801590610acd5750600160a060020a0381163014155b1515610ad857600080fd5b610ae28686611321565b1515610aed57600080fd5b85600160a060020a031633600160a060020a03167fe19260aff97b920c7df27010903aeb9c8d2be5d310a2c67824cf3f15396e4c16878787604051808481526020018060200182810382528484828181526020019250808284376040519201829003965090945050505050a3610b62866112bd565b15610bae57610ba333878787878080601f0160208091040260200160405190810160405280939291908181526020018383808284375061132d945050505050565b1515610bae57600080fd5b50600195945050505050565b600654600090600160a060020a03163314610bd457600080fd5b60065474010000000000000000000000000000000000000000900460ff1615610bfc57600080fd5b600454610c0f908363ffffffff6112d716565b600455600160a060020a038316600090815260036020526040902054610c3b908363ffffffff6112d716565b600160a060020a038416600081815260036020908152604091829020939093558051858152905191927f0f6798a560793a54c3bcfe86a93cde1e73087d944c0ea20544137d412139688592918290030190a2604080518381529051600160a060020a03851691600091600080516020611d6d8339815191529181900360200190a350600192915050565b610ccf33826114a7565b50565b60408051808201909152600181527f3100000000000000000000000000000000000000000000000000000000000000602082015281565b336000908152600560209081526040808320600160a060020a0386168452909152812054808310610d5d57336000908152600560209081526040808320600160a060020a0388168452909152812055610d92565b610d6d818463ffffffff6112c516565b336000908152600560209081526040808320600160a060020a03891684529091529020555b336000818152600560209081526040808320600160a060020a038916808552908352928190205481519081529051929392600080516020611d8d833981519152929181900390910190a35060019392505050565b600654600160a060020a03163314610dfd57600080fd5b610e078282611596565b5050565b600160a060020a031660009081526003602052604090205490565b600654600160a060020a031633146101b057600080fd5b600754600160a060020a0390811691161490565b6000806000fd5b60096020526000908152604090205481565b600260056000909192565b600654600160a060020a031681565b600080861580610e945750864211155b1515610e9f57600080fd5b604080517fea2aa0a1be11a07ed86d755c93467f4f82362b452371d1ba94d1715123511acb6020820152600160a060020a03808d16828401528b166060820152608081018a905260a0810189905287151560c0808301919091528251808303909101815260e0909101909152610f14906115d4565b9150610f22828686866116db565b600160a060020a038b8116911614610f3957600080fd5b600160a060020a038a1660009081526009602052604090208054600181019091558814610f6557600080fd5b85610f71576000610f75565b6000195b905085610f83576000610f85565b865b600160a060020a03808c166000908152600a60209081526040808320938e1683529290522055610fb68a8a836118e1565b50505050505050505050565b60018054604080516020600284861615610100026000190190941693909304601f810184900484028201840190925281815292918301828280156107ef5780601f106107c4576101008083540402835291602001916107ef565b60006110288383610d09565b9392505050565b600061103b8383611321565b151561104657600080fd5b6108043384846112ea565b61105c33838361086d565b505050565b61106c83838361086d565b50505050565b7fea2aa0a1be11a07ed86d755c93467f4f82362b452371d1ba94d1715123511acb81565b600754600160a060020a031690565b600080428610156110b557600080fd5b600160a060020a03808a1660008181526009602090815260409182902080546001810190915582517f6e71edae12b1b97f4d1f60370fef10105fa2faae0126114a169c64845d6126c99281019290925281830193909352928b166060840152608083018a905260a0830182905260c08084018a90528151808503909101815260e090930190529250611146906115d4565b9050611154818686866116db565b600160a060020a038a811691161461116b57600080fd5b61117689898961127b565b505050505050505050565b336000908152600560209081526040808320600160a060020a03861684529091528120546111b5908363ffffffff6112d716565b336000818152600560209081526040808320600160a060020a038916808552908352928190208590558051948552519193600080516020611d8d833981519152929081900390910190a350600192915050565b600160a060020a03918216600090815260056020908152604080832093909416825291909152205490565b61105c82338361086d565b600654600160a060020a0316331461125557600080fd5b610ccf81611a3c565b600a60209081526000928352604080842090915290825290205481565b6112868383836118e1565b60001981141561105c57600160a060020a038084166000908152600a60209081526040808320938616835292905290812055505050565b6000903b1190565b6000828211156112d157fe5b50900390565b818101828110156112e457fe5b92915050565b6112f382610e3d565b1561105c576040805160008152602081019091526113169084908490849061132d565b151561105c57600080fd5b60006110288383611aba565b600083600160a060020a031663a4c0ed3660e01b8685856040516024018084600160a060020a0316600160a060020a0316815260200183815260200180602001828103825283818151815260200191508051906020019080838360005b838110156113a257818101518382015260200161138a565b50505050905090810190601f1680156113cf5780820380516001836020036101000a031916815260200191505b5060408051601f198184030181529181526020820180517bffffffffffffffffffffffffffffffffffffffffffffffffffffffff167fffffffff00000000000000000000000000000000000000000000000000000000909916989098178852518151919790965086955093509150819050838360005b8381101561145d578181015183820152602001611445565b50505050905090810190601f16801561148a5780820380516001836020036101000a031916815260200191505b509150506000604051808303816000865af1979650505050505050565b600160a060020a0382166000908152600360205260409020548111156114cc57600080fd5b600160a060020a0382166000908152600360205260409020546114f5908263ffffffff6112c516565b600160a060020a038316600090815260036020526040902055600454611521908263ffffffff6112c516565b600455604080518281529051600160a060020a038416917fcc16f5dbb4873280815c1ee09dbd06736cffcc184412cf7a71a0fdb75d397ca5919081900360200190a2604080518281529051600091600160a060020a03851691600080516020611d6d8339815191529181900360200190a35050565b80600160a060020a03811615156115ac57600080fd5b600160a060020a03831615156115ca576115c582611b89565b61105c565b61105c8383611b95565b6000600854826040518082805190602001908083835b602083106116095780518252601f1990920191602091820191016115ea565b51815160209384036101000a6000190180199092169116179052604080519290940182900382207f190100000000000000000000000000000000000000000000000000000000000083830152602283019790975260428083019790975283518083039097018752606290910192839052855192945084935085019190508083835b602083106116a95780518252601f19909201916020918201910161168a565b5181516020939093036101000a6000190180199091169216919091179052604051920182900390912095945050505050565b6000808460ff16601b14806116f357508460ff16601c145b151561176f576040805160e560020a62461bcd02815260206004820152602260248201527f45434453413a20696e76616c6964207369676e6174757265202776272076616c60448201527f7565000000000000000000000000000000000000000000000000000000000000606482015290519081900360840190fd5b7f7fffffffffffffffffffffffffffffff5d576e7357a4501ddfe92f46681b20a0600084901c1115611811576040805160e560020a62461bcd02815260206004820152602260248201527f45434453413a20696e76616c6964207369676e6174757265202773272076616c60448201527f7565000000000000000000000000000000000000000000000000000000000000606482015290519081900360840190fd5b60408051600080825260208083018085528a905260ff8916838501526060830188905260808301879052925160019360a0808501949193601f19840193928390039091019190865af115801561186b573d6000803e3d6000fd5b5050604051601f190151915050600160a060020a03811615156118d8576040805160e560020a62461bcd02815260206004820152601860248201527f45434453413a20696e76616c6964207369676e61747572650000000000000000604482015290519081900360640190fd5b95945050505050565b600160a060020a0383161515611966576040805160e560020a62461bcd028152602060048201526024808201527f45524332303a20617070726f76652066726f6d20746865207a65726f2061646460448201527f7265737300000000000000000000000000000000000000000000000000000000606482015290519081900360840190fd5b600160a060020a03821615156119ec576040805160e560020a62461bcd02815260206004820152602260248201527f45524332303a20617070726f766520746f20746865207a65726f20616464726560448201527f7373000000000000000000000000000000000000000000000000000000000000606482015290519081900360840190fd5b600160a060020a0380841660008181526005602090815260408083209487168084529482529182902085905581518581529151600080516020611d8d8339815191529281900390910190a3505050565b600160a060020a0381161515611a5157600080fd5b600654604051600160a060020a038084169216907f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e090600090a36006805473ffffffffffffffffffffffffffffffffffffffff1916600160a060020a0392909216919091179055565b33600090815260036020526040812054821115611ad657600080fd5b600160a060020a0383161515611aeb57600080fd5b33600090815260036020526040902054611b0b908363ffffffff6112c516565b3360009081526003602052604080822092909255600160a060020a03851681522054611b3d908363ffffffff6112d716565b600160a060020a038416600081815260036020908152604091829020939093558051858152905191923392600080516020611d6d8339815191529281900390910190a350600192915050565b3031610e078282611c42565b604080517f70a0823100000000000000000000000000000000000000000000000000000000815230600482015290518391600091600160a060020a038416916370a0823191602480830192602092919082900301818787803b158015611bfa57600080fd5b505af1158015611c0e573d6000803e3d6000fd5b505050506040513d6020811015611c2457600080fd5b5051905061106c600160a060020a038516848363ffffffff611caa16565b604051600160a060020a0383169082156108fc029083906000818181858888f193505050501515610e07578082611c77611d3c565b600160a060020a039091168152604051908190036020019082f080158015611ca3573d6000803e3d6000fd5b5050505050565b82600160a060020a031663a9059cbb83836040518363ffffffff1660e01b81526004018083600160a060020a0316600160a060020a0316815260200182815260200192505050600060405180830381600087803b158015611d0a57600080fd5b505af1158015611d1e573d6000803e3d6000fd5b505050503d1561105c5760206000803e600051151561105c57600080fd5b604051602180611d4c833901905600608060405260405160208060218339810160405251600160a060020a038116ff00ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925a165627a7a7230582007f5a5e1d9fa6bdb682c6bd9421ecebd4f2b8795e082207e4f9620819b5199e30029" . parse () . expect ("invalid bytecode")
        });
    pub struct PermittableToken<M>(ethers::contract::Contract<M>);
    impl<M> Clone for PermittableToken<M> {
        fn clone(&self) -> Self {
            PermittableToken(self.0.clone())
        }
    }
    impl<M> std::ops::Deref for PermittableToken<M> {
        type Target = ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> std::fmt::Debug for PermittableToken<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(PermittableToken))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ethers::providers::Middleware> PermittableToken<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            ethers::contract::Contract::new(address.into(), PERMITTABLETOKEN_ABI.clone(), client)
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
                PERMITTABLETOKEN_ABI.clone(),
                PERMITTABLETOKEN_BYTECODE.clone().into(),
                client,
            );
            let deployer = factory.deploy(constructor_args)?;
            let deployer = ethers::contract::ContractDeployer::new(deployer);
            Ok(deployer)
        }
        #[doc = "Calls the contract's `DOMAIN_SEPARATOR` (0x3644e515) function"]
        pub fn domain_separator(&self) -> ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([54, 68, 229, 21], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `PERMIT_TYPEHASH` (0x30adf81f) function"]
        pub fn permit_typehash(&self) -> ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([48, 173, 248, 31], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `PERMIT_TYPEHASH_LEGACY` (0xc6a1dedf) function"]
        pub fn permit_typehash_legacy(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([198, 161, 222, 223], ())
                .expect("method not found (this should never happen)")
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
            to: ethers::core::types::Address,
            value: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([9, 94, 167, 179], (to, value))
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
        #[doc = "Calls the contract's `expirations` (0xff9e884d) function"]
        pub fn expirations(
            &self,
            p0: ethers::core::types::Address,
            p1: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([255, 158, 136, 77], (p0, p1))
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
            to: ethers::core::types::Address,
            added_value: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([57, 80, 147, 81], (to, added_value))
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
        #[doc = "Calls the contract's `move` (0xbb35783b) function"]
        pub fn move_(
            &self,
            from: ethers::core::types::Address,
            to: ethers::core::types::Address,
            amount: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([187, 53, 120, 59], (from, to, amount))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `name` (0x06fdde03) function"]
        pub fn name(&self) -> ethers::contract::builders::ContractCall<M, String> {
            self.0
                .method_hash([6, 253, 222, 3], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `nonces` (0x7ecebe00) function"]
        pub fn nonces(
            &self,
            p0: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([126, 206, 190, 0], p0)
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
        #[doc = "Calls the contract's `permit` (0x8fcbaf0c) function"]
        pub fn permit_with_nonce_and_expiry_and_allowed(
            &self,
            holder: ethers::core::types::Address,
            spender: ethers::core::types::Address,
            nonce: ethers::core::types::U256,
            expiry: ethers::core::types::U256,
            allowed: bool,
            v: u8,
            r: [u8; 32],
            s: [u8; 32],
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [143, 203, 175, 12],
                    (holder, spender, nonce, expiry, allowed, v, r, s),
                )
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `permit` (0xd505accf) function"]
        pub fn permit(
            &self,
            holder: ethers::core::types::Address,
            spender: ethers::core::types::Address,
            value: ethers::core::types::U256,
            deadline: ethers::core::types::U256,
            v: u8,
            r: [u8; 32],
            s: [u8; 32],
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [213, 5, 172, 207],
                    (holder, spender, value, deadline, v, r, s),
                )
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `pull` (0xf2d5d56b) function"]
        pub fn pull(
            &self,
            from: ethers::core::types::Address,
            amount: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([242, 213, 213, 107], (from, amount))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `push` (0xb753a98c) function"]
        pub fn push(
            &self,
            to: ethers::core::types::Address,
            amount: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([183, 83, 169, 140], (to, amount))
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
            sender: ethers::core::types::Address,
            recipient: ethers::core::types::Address,
            amount: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([35, 184, 114, 221], (sender, recipient, amount))
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
        #[doc = "Calls the contract's `version` (0x54fd4d50) function"]
        pub fn version(&self) -> ethers::contract::builders::ContractCall<M, String> {
            self.0
                .method_hash([84, 253, 77, 80], ())
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
        pub fn events(&self) -> ethers::contract::builders::Event<M, PermittableTokenEvents> {
            self.0.event_with_filter(Default::default())
        }
    }
    impl<M: ethers::providers::Middleware> From<ethers::contract::Contract<M>> for PermittableToken<M> {
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
    pub enum PermittableTokenEvents {
        ApprovalFilter(ApprovalFilter),
        BurnFilter(BurnFilter),
        MintFilter(MintFilter),
        MintFinishedFilter(MintFinishedFilter),
        OwnershipRenouncedFilter(OwnershipRenouncedFilter),
        OwnershipTransferredFilter(OwnershipTransferredFilter),
        Transfer1Filter(Transfer1Filter),
        Transfer2Filter(Transfer2Filter),
    }
    impl ethers::contract::EthLogDecode for PermittableTokenEvents {
        fn decode_log(
            log: &ethers::core::abi::RawLog,
        ) -> ::std::result::Result<Self, ethers::core::abi::Error>
        where
            Self: Sized,
        {
            if let Ok(decoded) = ApprovalFilter::decode_log(log) {
                return Ok(PermittableTokenEvents::ApprovalFilter(decoded));
            }
            if let Ok(decoded) = BurnFilter::decode_log(log) {
                return Ok(PermittableTokenEvents::BurnFilter(decoded));
            }
            if let Ok(decoded) = MintFilter::decode_log(log) {
                return Ok(PermittableTokenEvents::MintFilter(decoded));
            }
            if let Ok(decoded) = MintFinishedFilter::decode_log(log) {
                return Ok(PermittableTokenEvents::MintFinishedFilter(decoded));
            }
            if let Ok(decoded) = OwnershipRenouncedFilter::decode_log(log) {
                return Ok(PermittableTokenEvents::OwnershipRenouncedFilter(decoded));
            }
            if let Ok(decoded) = OwnershipTransferredFilter::decode_log(log) {
                return Ok(PermittableTokenEvents::OwnershipTransferredFilter(decoded));
            }
            if let Ok(decoded) = Transfer1Filter::decode_log(log) {
                return Ok(PermittableTokenEvents::Transfer1Filter(decoded));
            }
            if let Ok(decoded) = Transfer2Filter::decode_log(log) {
                return Ok(PermittableTokenEvents::Transfer2Filter(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData)
        }
    }
    impl ::std::fmt::Display for PermittableTokenEvents {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                PermittableTokenEvents::ApprovalFilter(element) => element.fmt(f),
                PermittableTokenEvents::BurnFilter(element) => element.fmt(f),
                PermittableTokenEvents::MintFilter(element) => element.fmt(f),
                PermittableTokenEvents::MintFinishedFilter(element) => element.fmt(f),
                PermittableTokenEvents::OwnershipRenouncedFilter(element) => element.fmt(f),
                PermittableTokenEvents::OwnershipTransferredFilter(element) => element.fmt(f),
                PermittableTokenEvents::Transfer1Filter(element) => element.fmt(f),
                PermittableTokenEvents::Transfer2Filter(element) => element.fmt(f),
            }
        }
    }
    #[doc = "Container type for all input parameters for the `DOMAIN_SEPARATOR` function with signature `DOMAIN_SEPARATOR()` and selector `[54, 68, 229, 21]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "DOMAIN_SEPARATOR", abi = "DOMAIN_SEPARATOR()")]
    pub struct DomainSeparatorCall;
    #[doc = "Container type for all input parameters for the `PERMIT_TYPEHASH` function with signature `PERMIT_TYPEHASH()` and selector `[48, 173, 248, 31]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "PERMIT_TYPEHASH", abi = "PERMIT_TYPEHASH()")]
    pub struct PermitTypehashCall;
    #[doc = "Container type for all input parameters for the `PERMIT_TYPEHASH_LEGACY` function with signature `PERMIT_TYPEHASH_LEGACY()` and selector `[198, 161, 222, 223]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "PERMIT_TYPEHASH_LEGACY", abi = "PERMIT_TYPEHASH_LEGACY()")]
    pub struct PermitTypehashLegacyCall;
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
        pub to: ethers::core::types::Address,
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
    #[doc = "Container type for all input parameters for the `expirations` function with signature `expirations(address,address)` and selector `[255, 158, 136, 77]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "expirations", abi = "expirations(address,address)")]
    pub struct ExpirationsCall(
        pub ethers::core::types::Address,
        pub ethers::core::types::Address,
    );
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
        pub to: ethers::core::types::Address,
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
    #[doc = "Container type for all input parameters for the `move` function with signature `move(address,address,uint256)` and selector `[187, 53, 120, 59]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "move", abi = "move(address,address,uint256)")]
    pub struct MoveCall {
        pub from: ethers::core::types::Address,
        pub to: ethers::core::types::Address,
        pub amount: ethers::core::types::U256,
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
    #[doc = "Container type for all input parameters for the `nonces` function with signature `nonces(address)` and selector `[126, 206, 190, 0]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "nonces", abi = "nonces(address)")]
    pub struct NoncesCall(pub ethers::core::types::Address);
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
    #[doc = "Container type for all input parameters for the `permit` function with signature `permit(address,address,uint256,uint256,bool,uint8,bytes32,bytes32)` and selector `[143, 203, 175, 12]`"]
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
        name = "permit",
        abi = "permit(address,address,uint256,uint256,bool,uint8,bytes32,bytes32)"
    )]
    pub struct PermitWithNonceAndExpiryAndAllowedCall {
        pub holder: ethers::core::types::Address,
        pub spender: ethers::core::types::Address,
        pub nonce: ethers::core::types::U256,
        pub expiry: ethers::core::types::U256,
        pub allowed: bool,
        pub v: u8,
        pub r: [u8; 32],
        pub s: [u8; 32],
    }
    #[doc = "Container type for all input parameters for the `permit` function with signature `permit(address,address,uint256,uint256,uint8,bytes32,bytes32)` and selector `[213, 5, 172, 207]`"]
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
        name = "permit",
        abi = "permit(address,address,uint256,uint256,uint8,bytes32,bytes32)"
    )]
    pub struct PermitCall {
        pub holder: ethers::core::types::Address,
        pub spender: ethers::core::types::Address,
        pub value: ethers::core::types::U256,
        pub deadline: ethers::core::types::U256,
        pub v: u8,
        pub r: [u8; 32],
        pub s: [u8; 32],
    }
    #[doc = "Container type for all input parameters for the `pull` function with signature `pull(address,uint256)` and selector `[242, 213, 213, 107]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "pull", abi = "pull(address,uint256)")]
    pub struct PullCall {
        pub from: ethers::core::types::Address,
        pub amount: ethers::core::types::U256,
    }
    #[doc = "Container type for all input parameters for the `push` function with signature `push(address,uint256)` and selector `[183, 83, 169, 140]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "push", abi = "push(address,uint256)")]
    pub struct PushCall {
        pub to: ethers::core::types::Address,
        pub amount: ethers::core::types::U256,
    }
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
        pub sender: ethers::core::types::Address,
        pub recipient: ethers::core::types::Address,
        pub amount: ethers::core::types::U256,
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
    #[doc = "Container type for all input parameters for the `version` function with signature `version()` and selector `[84, 253, 77, 80]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "version", abi = "version()")]
    pub struct VersionCall;
    #[derive(Debug, Clone, PartialEq, Eq, ethers :: contract :: EthAbiType)]
    pub enum PermittableTokenCalls {
        DomainSeparator(DomainSeparatorCall),
        PermitTypehash(PermitTypehashCall),
        PermitTypehashLegacy(PermitTypehashLegacyCall),
        Allowance(AllowanceCall),
        Approve(ApproveCall),
        BalanceOf(BalanceOfCall),
        BridgeContract(BridgeContractCall),
        Burn(BurnCall),
        ClaimTokens(ClaimTokensCall),
        Decimals(DecimalsCall),
        DecreaseAllowance(DecreaseAllowanceCall),
        DecreaseApproval(DecreaseApprovalCall),
        Expirations(ExpirationsCall),
        FinishMinting(FinishMintingCall),
        GetTokenInterfacesVersion(GetTokenInterfacesVersionCall),
        IncreaseAllowance(IncreaseAllowanceCall),
        IncreaseApproval(IncreaseApprovalCall),
        IsBridge(IsBridgeCall),
        Mint(MintCall),
        MintingFinished(MintingFinishedCall),
        Move(MoveCall),
        Name(NameCall),
        Nonces(NoncesCall),
        Owner(OwnerCall),
        PermitWithNonceAndExpiryAndAllowed(PermitWithNonceAndExpiryAndAllowedCall),
        Permit(PermitCall),
        Pull(PullCall),
        Push(PushCall),
        RenounceOwnership(RenounceOwnershipCall),
        SetBridgeContract(SetBridgeContractCall),
        Symbol(SymbolCall),
        TotalSupply(TotalSupplyCall),
        Transfer(TransferCall),
        TransferAndCall(TransferAndCallCall),
        TransferFrom(TransferFromCall),
        TransferOwnership(TransferOwnershipCall),
        Version(VersionCall),
    }
    impl ethers::core::abi::AbiDecode for PermittableTokenCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::std::result::Result<Self, ethers::core::abi::AbiError> {
            if let Ok(decoded) =
                <DomainSeparatorCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PermittableTokenCalls::DomainSeparator(decoded));
            }
            if let Ok(decoded) =
                <PermitTypehashCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PermittableTokenCalls::PermitTypehash(decoded));
            }
            if let Ok(decoded) =
                <PermitTypehashLegacyCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PermittableTokenCalls::PermitTypehashLegacy(decoded));
            }
            if let Ok(decoded) =
                <AllowanceCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PermittableTokenCalls::Allowance(decoded));
            }
            if let Ok(decoded) =
                <ApproveCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PermittableTokenCalls::Approve(decoded));
            }
            if let Ok(decoded) =
                <BalanceOfCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PermittableTokenCalls::BalanceOf(decoded));
            }
            if let Ok(decoded) =
                <BridgeContractCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PermittableTokenCalls::BridgeContract(decoded));
            }
            if let Ok(decoded) = <BurnCall as ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(PermittableTokenCalls::Burn(decoded));
            }
            if let Ok(decoded) =
                <ClaimTokensCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PermittableTokenCalls::ClaimTokens(decoded));
            }
            if let Ok(decoded) =
                <DecimalsCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PermittableTokenCalls::Decimals(decoded));
            }
            if let Ok(decoded) =
                <DecreaseAllowanceCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PermittableTokenCalls::DecreaseAllowance(decoded));
            }
            if let Ok(decoded) =
                <DecreaseApprovalCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PermittableTokenCalls::DecreaseApproval(decoded));
            }
            if let Ok(decoded) =
                <ExpirationsCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PermittableTokenCalls::Expirations(decoded));
            }
            if let Ok(decoded) =
                <FinishMintingCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PermittableTokenCalls::FinishMinting(decoded));
            }
            if let Ok(decoded) =
                <GetTokenInterfacesVersionCall as ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                )
            {
                return Ok(PermittableTokenCalls::GetTokenInterfacesVersion(decoded));
            }
            if let Ok(decoded) =
                <IncreaseAllowanceCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PermittableTokenCalls::IncreaseAllowance(decoded));
            }
            if let Ok(decoded) =
                <IncreaseApprovalCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PermittableTokenCalls::IncreaseApproval(decoded));
            }
            if let Ok(decoded) =
                <IsBridgeCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PermittableTokenCalls::IsBridge(decoded));
            }
            if let Ok(decoded) = <MintCall as ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(PermittableTokenCalls::Mint(decoded));
            }
            if let Ok(decoded) =
                <MintingFinishedCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PermittableTokenCalls::MintingFinished(decoded));
            }
            if let Ok(decoded) = <MoveCall as ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(PermittableTokenCalls::Move(decoded));
            }
            if let Ok(decoded) = <NameCall as ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(PermittableTokenCalls::Name(decoded));
            }
            if let Ok(decoded) = <NoncesCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PermittableTokenCalls::Nonces(decoded));
            }
            if let Ok(decoded) = <OwnerCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PermittableTokenCalls::Owner(decoded));
            }
            if let Ok(decoded) =
                <PermitWithNonceAndExpiryAndAllowedCall as ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                )
            {
                return Ok(PermittableTokenCalls::PermitWithNonceAndExpiryAndAllowed(
                    decoded,
                ));
            }
            if let Ok(decoded) = <PermitCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PermittableTokenCalls::Permit(decoded));
            }
            if let Ok(decoded) = <PullCall as ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(PermittableTokenCalls::Pull(decoded));
            }
            if let Ok(decoded) = <PushCall as ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(PermittableTokenCalls::Push(decoded));
            }
            if let Ok(decoded) =
                <RenounceOwnershipCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PermittableTokenCalls::RenounceOwnership(decoded));
            }
            if let Ok(decoded) =
                <SetBridgeContractCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PermittableTokenCalls::SetBridgeContract(decoded));
            }
            if let Ok(decoded) = <SymbolCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PermittableTokenCalls::Symbol(decoded));
            }
            if let Ok(decoded) =
                <TotalSupplyCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PermittableTokenCalls::TotalSupply(decoded));
            }
            if let Ok(decoded) =
                <TransferCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PermittableTokenCalls::Transfer(decoded));
            }
            if let Ok(decoded) =
                <TransferAndCallCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PermittableTokenCalls::TransferAndCall(decoded));
            }
            if let Ok(decoded) =
                <TransferFromCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PermittableTokenCalls::TransferFrom(decoded));
            }
            if let Ok(decoded) =
                <TransferOwnershipCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PermittableTokenCalls::TransferOwnership(decoded));
            }
            if let Ok(decoded) =
                <VersionCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(PermittableTokenCalls::Version(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ethers::core::abi::AbiEncode for PermittableTokenCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                PermittableTokenCalls::DomainSeparator(element) => element.encode(),
                PermittableTokenCalls::PermitTypehash(element) => element.encode(),
                PermittableTokenCalls::PermitTypehashLegacy(element) => element.encode(),
                PermittableTokenCalls::Allowance(element) => element.encode(),
                PermittableTokenCalls::Approve(element) => element.encode(),
                PermittableTokenCalls::BalanceOf(element) => element.encode(),
                PermittableTokenCalls::BridgeContract(element) => element.encode(),
                PermittableTokenCalls::Burn(element) => element.encode(),
                PermittableTokenCalls::ClaimTokens(element) => element.encode(),
                PermittableTokenCalls::Decimals(element) => element.encode(),
                PermittableTokenCalls::DecreaseAllowance(element) => element.encode(),
                PermittableTokenCalls::DecreaseApproval(element) => element.encode(),
                PermittableTokenCalls::Expirations(element) => element.encode(),
                PermittableTokenCalls::FinishMinting(element) => element.encode(),
                PermittableTokenCalls::GetTokenInterfacesVersion(element) => element.encode(),
                PermittableTokenCalls::IncreaseAllowance(element) => element.encode(),
                PermittableTokenCalls::IncreaseApproval(element) => element.encode(),
                PermittableTokenCalls::IsBridge(element) => element.encode(),
                PermittableTokenCalls::Mint(element) => element.encode(),
                PermittableTokenCalls::MintingFinished(element) => element.encode(),
                PermittableTokenCalls::Move(element) => element.encode(),
                PermittableTokenCalls::Name(element) => element.encode(),
                PermittableTokenCalls::Nonces(element) => element.encode(),
                PermittableTokenCalls::Owner(element) => element.encode(),
                PermittableTokenCalls::PermitWithNonceAndExpiryAndAllowed(element) => {
                    element.encode()
                }
                PermittableTokenCalls::Permit(element) => element.encode(),
                PermittableTokenCalls::Pull(element) => element.encode(),
                PermittableTokenCalls::Push(element) => element.encode(),
                PermittableTokenCalls::RenounceOwnership(element) => element.encode(),
                PermittableTokenCalls::SetBridgeContract(element) => element.encode(),
                PermittableTokenCalls::Symbol(element) => element.encode(),
                PermittableTokenCalls::TotalSupply(element) => element.encode(),
                PermittableTokenCalls::Transfer(element) => element.encode(),
                PermittableTokenCalls::TransferAndCall(element) => element.encode(),
                PermittableTokenCalls::TransferFrom(element) => element.encode(),
                PermittableTokenCalls::TransferOwnership(element) => element.encode(),
                PermittableTokenCalls::Version(element) => element.encode(),
            }
        }
    }
    impl ::std::fmt::Display for PermittableTokenCalls {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                PermittableTokenCalls::DomainSeparator(element) => element.fmt(f),
                PermittableTokenCalls::PermitTypehash(element) => element.fmt(f),
                PermittableTokenCalls::PermitTypehashLegacy(element) => element.fmt(f),
                PermittableTokenCalls::Allowance(element) => element.fmt(f),
                PermittableTokenCalls::Approve(element) => element.fmt(f),
                PermittableTokenCalls::BalanceOf(element) => element.fmt(f),
                PermittableTokenCalls::BridgeContract(element) => element.fmt(f),
                PermittableTokenCalls::Burn(element) => element.fmt(f),
                PermittableTokenCalls::ClaimTokens(element) => element.fmt(f),
                PermittableTokenCalls::Decimals(element) => element.fmt(f),
                PermittableTokenCalls::DecreaseAllowance(element) => element.fmt(f),
                PermittableTokenCalls::DecreaseApproval(element) => element.fmt(f),
                PermittableTokenCalls::Expirations(element) => element.fmt(f),
                PermittableTokenCalls::FinishMinting(element) => element.fmt(f),
                PermittableTokenCalls::GetTokenInterfacesVersion(element) => element.fmt(f),
                PermittableTokenCalls::IncreaseAllowance(element) => element.fmt(f),
                PermittableTokenCalls::IncreaseApproval(element) => element.fmt(f),
                PermittableTokenCalls::IsBridge(element) => element.fmt(f),
                PermittableTokenCalls::Mint(element) => element.fmt(f),
                PermittableTokenCalls::MintingFinished(element) => element.fmt(f),
                PermittableTokenCalls::Move(element) => element.fmt(f),
                PermittableTokenCalls::Name(element) => element.fmt(f),
                PermittableTokenCalls::Nonces(element) => element.fmt(f),
                PermittableTokenCalls::Owner(element) => element.fmt(f),
                PermittableTokenCalls::PermitWithNonceAndExpiryAndAllowed(element) => {
                    element.fmt(f)
                }
                PermittableTokenCalls::Permit(element) => element.fmt(f),
                PermittableTokenCalls::Pull(element) => element.fmt(f),
                PermittableTokenCalls::Push(element) => element.fmt(f),
                PermittableTokenCalls::RenounceOwnership(element) => element.fmt(f),
                PermittableTokenCalls::SetBridgeContract(element) => element.fmt(f),
                PermittableTokenCalls::Symbol(element) => element.fmt(f),
                PermittableTokenCalls::TotalSupply(element) => element.fmt(f),
                PermittableTokenCalls::Transfer(element) => element.fmt(f),
                PermittableTokenCalls::TransferAndCall(element) => element.fmt(f),
                PermittableTokenCalls::TransferFrom(element) => element.fmt(f),
                PermittableTokenCalls::TransferOwnership(element) => element.fmt(f),
                PermittableTokenCalls::Version(element) => element.fmt(f),
            }
        }
    }
    impl ::std::convert::From<DomainSeparatorCall> for PermittableTokenCalls {
        fn from(var: DomainSeparatorCall) -> Self {
            PermittableTokenCalls::DomainSeparator(var)
        }
    }
    impl ::std::convert::From<PermitTypehashCall> for PermittableTokenCalls {
        fn from(var: PermitTypehashCall) -> Self {
            PermittableTokenCalls::PermitTypehash(var)
        }
    }
    impl ::std::convert::From<PermitTypehashLegacyCall> for PermittableTokenCalls {
        fn from(var: PermitTypehashLegacyCall) -> Self {
            PermittableTokenCalls::PermitTypehashLegacy(var)
        }
    }
    impl ::std::convert::From<AllowanceCall> for PermittableTokenCalls {
        fn from(var: AllowanceCall) -> Self {
            PermittableTokenCalls::Allowance(var)
        }
    }
    impl ::std::convert::From<ApproveCall> for PermittableTokenCalls {
        fn from(var: ApproveCall) -> Self {
            PermittableTokenCalls::Approve(var)
        }
    }
    impl ::std::convert::From<BalanceOfCall> for PermittableTokenCalls {
        fn from(var: BalanceOfCall) -> Self {
            PermittableTokenCalls::BalanceOf(var)
        }
    }
    impl ::std::convert::From<BridgeContractCall> for PermittableTokenCalls {
        fn from(var: BridgeContractCall) -> Self {
            PermittableTokenCalls::BridgeContract(var)
        }
    }
    impl ::std::convert::From<BurnCall> for PermittableTokenCalls {
        fn from(var: BurnCall) -> Self {
            PermittableTokenCalls::Burn(var)
        }
    }
    impl ::std::convert::From<ClaimTokensCall> for PermittableTokenCalls {
        fn from(var: ClaimTokensCall) -> Self {
            PermittableTokenCalls::ClaimTokens(var)
        }
    }
    impl ::std::convert::From<DecimalsCall> for PermittableTokenCalls {
        fn from(var: DecimalsCall) -> Self {
            PermittableTokenCalls::Decimals(var)
        }
    }
    impl ::std::convert::From<DecreaseAllowanceCall> for PermittableTokenCalls {
        fn from(var: DecreaseAllowanceCall) -> Self {
            PermittableTokenCalls::DecreaseAllowance(var)
        }
    }
    impl ::std::convert::From<DecreaseApprovalCall> for PermittableTokenCalls {
        fn from(var: DecreaseApprovalCall) -> Self {
            PermittableTokenCalls::DecreaseApproval(var)
        }
    }
    impl ::std::convert::From<ExpirationsCall> for PermittableTokenCalls {
        fn from(var: ExpirationsCall) -> Self {
            PermittableTokenCalls::Expirations(var)
        }
    }
    impl ::std::convert::From<FinishMintingCall> for PermittableTokenCalls {
        fn from(var: FinishMintingCall) -> Self {
            PermittableTokenCalls::FinishMinting(var)
        }
    }
    impl ::std::convert::From<GetTokenInterfacesVersionCall> for PermittableTokenCalls {
        fn from(var: GetTokenInterfacesVersionCall) -> Self {
            PermittableTokenCalls::GetTokenInterfacesVersion(var)
        }
    }
    impl ::std::convert::From<IncreaseAllowanceCall> for PermittableTokenCalls {
        fn from(var: IncreaseAllowanceCall) -> Self {
            PermittableTokenCalls::IncreaseAllowance(var)
        }
    }
    impl ::std::convert::From<IncreaseApprovalCall> for PermittableTokenCalls {
        fn from(var: IncreaseApprovalCall) -> Self {
            PermittableTokenCalls::IncreaseApproval(var)
        }
    }
    impl ::std::convert::From<IsBridgeCall> for PermittableTokenCalls {
        fn from(var: IsBridgeCall) -> Self {
            PermittableTokenCalls::IsBridge(var)
        }
    }
    impl ::std::convert::From<MintCall> for PermittableTokenCalls {
        fn from(var: MintCall) -> Self {
            PermittableTokenCalls::Mint(var)
        }
    }
    impl ::std::convert::From<MintingFinishedCall> for PermittableTokenCalls {
        fn from(var: MintingFinishedCall) -> Self {
            PermittableTokenCalls::MintingFinished(var)
        }
    }
    impl ::std::convert::From<MoveCall> for PermittableTokenCalls {
        fn from(var: MoveCall) -> Self {
            PermittableTokenCalls::Move(var)
        }
    }
    impl ::std::convert::From<NameCall> for PermittableTokenCalls {
        fn from(var: NameCall) -> Self {
            PermittableTokenCalls::Name(var)
        }
    }
    impl ::std::convert::From<NoncesCall> for PermittableTokenCalls {
        fn from(var: NoncesCall) -> Self {
            PermittableTokenCalls::Nonces(var)
        }
    }
    impl ::std::convert::From<OwnerCall> for PermittableTokenCalls {
        fn from(var: OwnerCall) -> Self {
            PermittableTokenCalls::Owner(var)
        }
    }
    impl ::std::convert::From<PermitWithNonceAndExpiryAndAllowedCall> for PermittableTokenCalls {
        fn from(var: PermitWithNonceAndExpiryAndAllowedCall) -> Self {
            PermittableTokenCalls::PermitWithNonceAndExpiryAndAllowed(var)
        }
    }
    impl ::std::convert::From<PermitCall> for PermittableTokenCalls {
        fn from(var: PermitCall) -> Self {
            PermittableTokenCalls::Permit(var)
        }
    }
    impl ::std::convert::From<PullCall> for PermittableTokenCalls {
        fn from(var: PullCall) -> Self {
            PermittableTokenCalls::Pull(var)
        }
    }
    impl ::std::convert::From<PushCall> for PermittableTokenCalls {
        fn from(var: PushCall) -> Self {
            PermittableTokenCalls::Push(var)
        }
    }
    impl ::std::convert::From<RenounceOwnershipCall> for PermittableTokenCalls {
        fn from(var: RenounceOwnershipCall) -> Self {
            PermittableTokenCalls::RenounceOwnership(var)
        }
    }
    impl ::std::convert::From<SetBridgeContractCall> for PermittableTokenCalls {
        fn from(var: SetBridgeContractCall) -> Self {
            PermittableTokenCalls::SetBridgeContract(var)
        }
    }
    impl ::std::convert::From<SymbolCall> for PermittableTokenCalls {
        fn from(var: SymbolCall) -> Self {
            PermittableTokenCalls::Symbol(var)
        }
    }
    impl ::std::convert::From<TotalSupplyCall> for PermittableTokenCalls {
        fn from(var: TotalSupplyCall) -> Self {
            PermittableTokenCalls::TotalSupply(var)
        }
    }
    impl ::std::convert::From<TransferCall> for PermittableTokenCalls {
        fn from(var: TransferCall) -> Self {
            PermittableTokenCalls::Transfer(var)
        }
    }
    impl ::std::convert::From<TransferAndCallCall> for PermittableTokenCalls {
        fn from(var: TransferAndCallCall) -> Self {
            PermittableTokenCalls::TransferAndCall(var)
        }
    }
    impl ::std::convert::From<TransferFromCall> for PermittableTokenCalls {
        fn from(var: TransferFromCall) -> Self {
            PermittableTokenCalls::TransferFrom(var)
        }
    }
    impl ::std::convert::From<TransferOwnershipCall> for PermittableTokenCalls {
        fn from(var: TransferOwnershipCall) -> Self {
            PermittableTokenCalls::TransferOwnership(var)
        }
    }
    impl ::std::convert::From<VersionCall> for PermittableTokenCalls {
        fn from(var: VersionCall) -> Self {
            PermittableTokenCalls::Version(var)
        }
    }
    #[doc = "Container type for all return fields from the `DOMAIN_SEPARATOR` function with signature `DOMAIN_SEPARATOR()` and selector `[54, 68, 229, 21]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct DomainSeparatorReturn(pub [u8; 32]);
    #[doc = "Container type for all return fields from the `PERMIT_TYPEHASH` function with signature `PERMIT_TYPEHASH()` and selector `[48, 173, 248, 31]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct PermitTypehashReturn(pub [u8; 32]);
    #[doc = "Container type for all return fields from the `PERMIT_TYPEHASH_LEGACY` function with signature `PERMIT_TYPEHASH_LEGACY()` and selector `[198, 161, 222, 223]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct PermitTypehashLegacyReturn(pub [u8; 32]);
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
    pub struct ApproveReturn {
        pub result: bool,
    }
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
    #[doc = "Container type for all return fields from the `expirations` function with signature `expirations(address,address)` and selector `[255, 158, 136, 77]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct ExpirationsReturn(pub ethers::core::types::U256);
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
    pub struct IncreaseAllowanceReturn {
        pub result: bool,
    }
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
    #[doc = "Container type for all return fields from the `nonces` function with signature `nonces(address)` and selector `[126, 206, 190, 0]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct NoncesReturn(pub ethers::core::types::U256);
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
    #[doc = "Container type for all return fields from the `version` function with signature `version()` and selector `[84, 253, 77, 80]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct VersionReturn(pub String);
}
