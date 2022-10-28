pub use hopr_distributor::*;
#[allow(clippy::too_many_arguments, non_camel_case_types)]
pub mod hopr_distributor {
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
    #[doc = "HoprDistributor was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    # [rustfmt :: skip] const __ABI : & str = "[{\"inputs\":[{\"internalType\":\"contract HoprToken\",\"name\":\"_token\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint128\",\"name\":\"_startTime\",\"type\":\"uint128\",\"components\":[]},{\"internalType\":\"uint128\",\"name\":\"_maxMintAmount\",\"type\":\"uint128\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"constructor\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"uint128\",\"name\":\"amount\",\"type\":\"uint128\",\"components\":[],\"indexed\":false},{\"internalType\":\"string\",\"name\":\"scheduleName\",\"type\":\"string\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"AllocationAdded\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"uint128\",\"name\":\"amount\",\"type\":\"uint128\",\"components\":[],\"indexed\":false},{\"internalType\":\"string\",\"name\":\"scheduleName\",\"type\":\"string\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"Claimed\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"previousOwner\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"newOwner\",\"type\":\"address\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"OwnershipTransferred\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"uint128[]\",\"name\":\"durations\",\"type\":\"uint128[]\",\"components\":[],\"indexed\":false},{\"internalType\":\"uint128[]\",\"name\":\"percents\",\"type\":\"uint128[]\",\"components\":[],\"indexed\":false},{\"internalType\":\"string\",\"name\":\"name\",\"type\":\"string\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"ScheduleAdded\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"MULTIPLIER\",\"outputs\":[{\"internalType\":\"uint128\",\"name\":\"\",\"type\":\"uint128\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address[]\",\"name\":\"accounts\",\"type\":\"address[]\",\"components\":[]},{\"internalType\":\"uint128[]\",\"name\":\"amounts\",\"type\":\"uint128[]\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"scheduleName\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"addAllocations\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"uint128[]\",\"name\":\"durations\",\"type\":\"uint128[]\",\"components\":[]},{\"internalType\":\"uint128[]\",\"name\":\"percents\",\"type\":\"uint128[]\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"name\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"addSchedule\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"allocations\",\"outputs\":[{\"internalType\":\"uint128\",\"name\":\"amount\",\"type\":\"uint128\",\"components\":[]},{\"internalType\":\"uint128\",\"name\":\"claimed\",\"type\":\"uint128\",\"components\":[]},{\"internalType\":\"uint128\",\"name\":\"lastClaim\",\"type\":\"uint128\",\"components\":[]},{\"internalType\":\"bool\",\"name\":\"revoked\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"scheduleName\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"claim\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"scheduleName\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"claimFor\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"scheduleName\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"getClaimable\",\"outputs\":[{\"internalType\":\"uint128\",\"name\":\"\",\"type\":\"uint128\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"name\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"getSchedule\",\"outputs\":[{\"internalType\":\"uint128[]\",\"name\":\"\",\"type\":\"uint128[]\",\"components\":[]},{\"internalType\":\"uint128[]\",\"name\":\"\",\"type\":\"uint128[]\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"maxMintAmount\",\"outputs\":[{\"internalType\":\"uint128\",\"name\":\"\",\"type\":\"uint128\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"owner\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"renounceOwnership\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"scheduleName\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"revokeAccount\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"startTime\",\"outputs\":[{\"internalType\":\"uint128\",\"name\":\"\",\"type\":\"uint128\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"token\",\"outputs\":[{\"internalType\":\"contract HoprToken\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"totalMinted\",\"outputs\":[{\"internalType\":\"uint128\",\"name\":\"\",\"type\":\"uint128\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"totalToBeMinted\",\"outputs\":[{\"internalType\":\"uint128\",\"name\":\"\",\"type\":\"uint128\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"newOwner\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"transferOwnership\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"uint128\",\"name\":\"_startTime\",\"type\":\"uint128\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"updateStartTime\",\"outputs\":[]}]" ;
    #[doc = r" The parsed JSON-ABI of the contract."]
    pub static HOPRDISTRIBUTOR_ABI: ethers::contract::Lazy<ethers::core::abi::Abi> =
        ethers::contract::Lazy::new(|| {
            ethers::core::utils::__serde_json::from_str(__ABI).expect("invalid abi")
        });
    #[doc = r" Bytecode of the #name contract"]
    pub static HOPRDISTRIBUTOR_BYTECODE: ethers::contract::Lazy<ethers::core::types::Bytes> =
        ethers::contract::Lazy::new(|| {
            "0x6080604052600060015534801561001557600080fd5b506040516121d53803806121d58339818101604052606081101561003857600080fd5b508051602082015160409092015190919060006100536100ee565b600080546001600160a01b0319166001600160a01b0383169081178255604051929350917f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e0908290a350600280546001600160801b03199081166001600160801b0394851617909155600380546001600160a01b0319166001600160a01b0395909516949094179093556004805490931691161790556100f2565b3390565b6120d4806101016000396000f3fe608060405234801561001057600080fd5b50600436106101165760003560e01c80638da5cb5b116100a2578063e574297011610071578063e5742970146105ad578063e82330b2146106bb578063f2fde38b14610739578063f3fe12c91461075f578063fc0c546a146107cd57610116565b80638da5cb5b14610472578063a2309ff814610496578063b733f67d1461049e578063c31cd7d7146104c457610116565b80632c902c7c116100e95780632c902c7c1461025f57806370a42898146102dd578063715018a61461035b57806372840f0e1461036357806378e979251461046a57610116565b80630373a3641461011b578063059f8b161461013f57806322bccfd714610147578063239c70ae14610257575b600080fd5b6101236107d5565b604080516001600160801b039092168252519081900360200190f35b6101236107eb565b6102556004803603606081101561015d57600080fd5b810190602081018135600160201b81111561017757600080fd5b82018360208201111561018957600080fd5b803590602001918460208302840111600160201b831117156101aa57600080fd5b919390929091602081019035600160201b8111156101c757600080fd5b8201836020820111156101d957600080fd5b803590602001918460208302840111600160201b831117156101fa57600080fd5b919390929091602081019035600160201b81111561021757600080fd5b82018360208201111561022957600080fd5b803590602001918460018302840111600160201b8311171561024a57600080fd5b5090925090506107f2565b005b610123610b9c565b6102556004803603604081101561027557600080fd5b6001600160a01b038235169190810190604081016020820135600160201b81111561029f57600080fd5b8201836020820111156102b157600080fd5b803590602001918460018302840111600160201b831117156102d257600080fd5b509092509050610bab565b610123600480360360408110156102f357600080fd5b6001600160a01b038235169190810190604081016020820135600160201b81111561031d57600080fd5b82018360208201111561032f57600080fd5b803590602001918460018302840111600160201b8311171561035057600080fd5b509092509050610d4f565b610255610dd2565b6103d16004803603602081101561037957600080fd5b810190602081018135600160201b81111561039357600080fd5b8201836020820111156103a557600080fd5b803590602001918460018302840111600160201b831117156103c657600080fd5b509092509050610e74565b604051808060200180602001838103835285818151815260200191508051906020019060200280838360005b838110156104155781810151838201526020016103fd565b50505050905001838103825284818151815260200191508051906020019060200280838360005b8381101561045457818101518382015260200161043c565b5050505090500194505050505060405180910390f35b610123610fd2565b61047a610fe1565b604080516001600160a01b039092168252519081900360200190f35b610123610ff0565b610255600480360360208110156104b457600080fd5b50356001600160801b0316610fff565b610578600480360360408110156104da57600080fd5b6001600160a01b038235169190810190604081016020820135600160201b81111561050457600080fd5b82018360208201111561051657600080fd5b803590602001918460018302840111600160201b8311171561053757600080fd5b91908080601f0160208091040260200160405190810160405280939291908181526020018383808284376000920191909152509295506110cd945050505050565b604080516001600160801b03958616815293851660208501529190931682820152911515606082015290519081900360800190f35b610255600480360360608110156105c357600080fd5b810190602081018135600160201b8111156105dd57600080fd5b8201836020820111156105ef57600080fd5b803590602001918460208302840111600160201b8311171561061057600080fd5b919390929091602081019035600160201b81111561062d57600080fd5b82018360208201111561063f57600080fd5b803590602001918460208302840111600160201b8311171561066057600080fd5b919390929091602081019035600160201b81111561067d57600080fd5b82018360208201111561068f57600080fd5b803590602001918460018302840111600160201b831117156106b057600080fd5b509092509050611122565b610255600480360360408110156106d157600080fd5b6001600160a01b038235169190810190604081016020820135600160201b8111156106fb57600080fd5b82018360208201111561070d57600080fd5b803590602001918460018302840111600160201b8311171561072e57600080fd5b509092509050611538565b6102556004803603602081101561074f57600080fd5b50356001600160a01b031661157d565b6102556004803603602081101561077557600080fd5b810190602081018135600160201b81111561078f57600080fd5b8201836020820111156107a157600080fd5b803590602001918460018302840111600160201b831117156107c257600080fd5b509092509050611675565b61047a6116b9565b600154600160801b90046001600160801b031681565b620f424081565b6107fa6116c8565b6000546001600160a01b0390811691161461084a576040805162461bcd60e51b81526020600482018190526024820152600080516020612059833981519152604482015290519081900360640190fd5b60058282604051808383808284379190910194855250506040519283900360200190922054151591506108bc9050576040805162461bcd60e51b815260206004820152601360248201527214d8da19591d5b19481b5d5cdd08195e1a5cdd606a1b604482015290519081900360640190fd5b8483146108fa5760405162461bcd60e51b815260040180806020018281038252602b815260200180611fd0602b913960400191505060405180910390fd5b600154600160801b90046001600160801b031660005b86811015610b76576006600089898481811061092857fe5b905060200201356001600160a01b03166001600160a01b03166001600160a01b0316815260200190815260200160002084846040518083838082843791909101948552505060405192839003602001909220546001600160801b03161591506109da9050576040805162461bcd60e51b815260206004820152601960248201527f416c6c6f636174696f6e206d757374206e6f7420657869737400000000000000604482015290519081900360640190fd5b8585828181106109e657fe5b905060200201356001600160801b0316600660008a8a85818110610a0657fe5b905060200201356001600160a01b03166001600160a01b03166001600160a01b031681526020019081526020016000208585604051808383808284379190910194855250506040519283900360200190922080546001600160801b03949094166001600160801b03199094169390931790925550610aa1905082878784818110610a8c57fe5b905060200201356001600160801b03166116cc565b6004549092506001600160801b039081169083161115610abd57fe5b878782818110610ac957fe5b905060200201356001600160a01b03166001600160a01b03167f499c64a5e7cdaa8e72ac9f0f4b080fce7a37c5ef24b2a9f67c7c6a728f5aec09878784818110610b0f57fe5b905060200201356001600160801b0316868660405180846001600160801b03168152602001806020018281038252848482818152602001925080828437600083820152604051601f909101601f1916909201829003965090945050505050a2600101610910565b50600180546001600160801b03928316600160801b029216919091179055505050505050565b6004546001600160801b031681565b610bb36116c8565b6000546001600160a01b03908116911614610c03576040805162461bcd60e51b81526020600482018190526024820152600080516020612059833981519152604482015290519081900360640190fd5b6001600160a01b038316600090815260066020526040808220905184908490808383808284379190910194855250506040519283900360200190922080549093506001600160801b031615159150610c9c9050576040805162461bcd60e51b8152602060048201526015602482015274105b1b1bd8d85d1a5bdb881b5d5cdd08195e1a5cdd605a1b604482015290519081900360640190fd5b6001810154600160801b900460ff1615610ce75760405162461bcd60e51b81526004018080602001828103825260268152602001806120796026913960400191505060405180910390fd5b6001818101805460ff60801b1916600160801b90811790915590548254610d2c926001600160801b0392819004831692610d27928082169290041661173b565b61173b565b600180546001600160801b03928316600160801b02921691909117905550505050565b6000610dca60058484604051808383808284378083019250505092505050908152602001604051809103902060066000876001600160a01b03166001600160a01b03168152602001908152602001600020858560405180838380828437808301925050509250505090815260200160405180910390206117aa565b949350505050565b610dda6116c8565b6000546001600160a01b03908116911614610e2a576040805162461bcd60e51b81526020600482018190526024820152600080516020612059833981519152604482015290519081900360640190fd5b600080546040516001600160a01b03909116907f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e0908390a3600080546001600160a01b0319169055565b60608060058484604051808383808284379190910194855250506040519283900360200183209260059250879150869080838380828437919091019485525050604080519384900360209081018520865480830287018301909352828652600101949350859250830182828015610f3c57602002820191906000526020600020906000905b82829054906101000a90046001600160801b03166001600160801b031681526020019060100190602082600f01049283019260010382029150808411610ef95790505b5050505050915080805480602002602001604051908101604052809291908181526020018280548015610fc057602002820191906000526020600020906000905b82829054906101000a90046001600160801b03166001600160801b031681526020019060100190602082600f01049283019260010382029150808411610f7d5790505b50505050509050915091509250929050565b6002546001600160801b031681565b6000546001600160a01b031690565b6001546001600160801b031681565b6110076116c8565b6000546001600160a01b03908116911614611057576040805162461bcd60e51b81526020600482018190526024820152600080516020612059833981519152604482015290519081900360640190fd5b61105f611971565b6002546001600160801b039182169116116110ab5760405162461bcd60e51b8152600401808060200182810382526027815260200180611ffb6027913960400191505060405180910390fd5b600280546001600160801b0319166001600160801b0392909216919091179055565b60066020908152600092835260409092208151808301840180519281529084019290930191909120915280546001909101546001600160801b0380831692600160801b90819004821692918216910460ff1684565b61112a6116c8565b6000546001600160a01b0390811691161461117a576040805162461bcd60e51b81526020600482018190526024820152600080516020612059833981519152604482015290519081900360640190fd5b600582826040518083838082843791909101948552505060405192839003602001909220541591506111f59050576040805162461bcd60e51b815260206004820152601760248201527f5363686564756c65206d757374206e6f74206578697374000000000000000000604482015290519081900360640190fd5b8483146112335760405162461bcd60e51b815260040180806020018281038252602d815260200180611f7d602d913960400191505060405180910390fd5b60008060005b878110156113535788888281811061124d57fe5b905060200201356001600160801b03166001600160801b0316836001600160801b0316106112ac5760405162461bcd60e51b815260040180806020018281038252602a815260200180611f2d602a913960400191505060405180910390fd5b8888828181106112b857fe5b905060200201356001600160801b03169250620f42406001600160801b03168787838181106112e357fe5b905060200201356001600160801b03166001600160801b031611156113395760405162461bcd60e51b81526004018080602001828103825260378152602001806120226037913960400191505060405180910390fd5b61134982888884818110610a8c57fe5b9150600101611239565b506001600160801b038116620f42401461139e5760405162461bcd60e51b8152600401808060200182810382526026815260200180611faa6026913960400191505060405180910390fd5b604051806040016040528089898080602002602001604051908101604052809392919081815260200183836020028082843760009201919091525050509082525060408051602089810282810182019093528982529283019290918a918a918291850190849080828437600092019190915250505091525060405160059086908690808383808284379190910194855250506040516020938190038401902084518051919461145294508593500190611e55565b50602082810151805161146b9260018501920190611e55565b509050507f14a9427471da7dbb7ecca56162a326853031d8fab46a74ab7b1b797591d9e4688888888888886040518080602001806020018060200184810384528a8a82818152602001925060200280828437600083820152601f01601f19169091018581038452888152602090810191508990890280828437600083820152601f01601f191690910185810383528681526020019050868680828437600083820152604051601f909101601f19169092018290039b50909950505050505050505050a15050505050505050565b6115788383838080601f01602080910402602001604051908101604052809392919081815260200183838082843760009201919091525061197592505050565b505050565b6115856116c8565b6000546001600160a01b039081169116146115d5576040805162461bcd60e51b81526020600482018190526024820152600080516020612059833981519152604482015290519081900360640190fd5b6001600160a01b03811661161a5760405162461bcd60e51b8152600401808060200182810382526026815260200180611f576026913960400191505060405180910390fd5b600080546040516001600160a01b03808516939216917f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e091a3600080546001600160a01b0319166001600160a01b0392909216919091179055565b6116b53383838080601f01602080910402602001604051908101604052809392919081815260200183838082843760009201919091525061197592505050565b5050565b6003546001600160a01b031681565b3390565b60008282016001600160801b038085169082161015611732576040805162461bcd60e51b815260206004820152601960248201527f75696e74313238206164646974696f6e206f766572666c6f7700000000000000604482015290519081900360640190fd5b90505b92915050565b6000826001600160801b0316826001600160801b031611156117a4576040805162461bcd60e51b815260206004820152601c60248201527f75696e74313238207375627472616374696f6e206f766572666c6f7700000000604482015290519081900360640190fd5b50900390565b60006117b4611971565b60025484546001600160801b039283169261180492169086906000906117d657fe5b90600052602060002090600291828204019190066010029054906101000a90046001600160801b03166116cc565b6001600160801b0316111561181b57506000611735565b611823611971565b60025484546001600160801b039283169261184a921690869060001981019081106117d657fe5b6001600160801b0316101561187f578154611878906001600160801b0380821691600160801b90041661173b565b9050611735565b6000805b84548110156119695760025485546000916118b0916001600160801b03909116908890859081106117d657fe5b90506118ba611971565b6001600160801b0316816001600160801b031611156118d95750611969565b60018501546001600160801b038083169116106118f65750611961565b845460018701805461195d9286926119589261194f926001600160801b031691908890811061192157fe5b90600052602060002090600291828204019190066010029054906101000a90046001600160801b0316611d40565b620f4240611dd0565b6116cc565b9250505b600101611883565b509392505050565b4290565b6001600160a01b0382166000908152600660209081526040808320905184519192859282918401908083835b602083106119c05780518252601f1990920191602091820191016119a1565b51815160209384036101000a6000190180199092169116179052920194855250604051938490030190922080549093506001600160801b031615159150611a509050576040805162461bcd60e51b815260206004820152601960248201527f5468657265206973206e6f7468696e6720746f20636c61696d00000000000000604482015290519081900360640190fd5b6001810154600160801b900460ff1615611aa6576040805162461bcd60e51b81526020600482015260126024820152711058d8dbdd5b9d081a5cc81c995d9bdad95960721b604482015290519081900360640190fd5b60006005836040518082805190602001908083835b60208310611ada5780518252601f199092019160209182019101611abb565b51815160209384036101000a60001901801990921691161790529201948552506040519384900301909220925060009150611b17905082846117aa565b83549091506001600160801b039081169082161115611b3257fe5b8254600090611b5190600160801b90046001600160801b0316836116cc565b84549091506001600160801b039081169082161115611b6c57fe5b600154600090611b85906001600160801b0316846116cc565b6004549091506001600160801b039081169082161115611ba157fe5b600180546001600160801b038084166001600160801b0319909216919091179091558554838216600160801b029116178555611bdb611971565b6001860180546001600160801b0319166001600160801b0392831617905560035460408051630dcdc7dd60e41b81526001600160a01b038b8116600483015293871660248201526080604482015260006084820181905260c0606483015260c482018190529151939092169263dcdc7dd09261010480820193929182900301818387803b158015611c6b57600080fd5b505af1158015611c7f573d6000803e3d6000fd5b50505050866001600160a01b03167fd6d52022b5ae5ce877753d56a79a1299605b05220771f26b0817599cabd2b6b4848860405180836001600160801b0316815260200180602001828103825283818151815260200191508051906020019080838360005b83811015611cfc578181015183820152602001611ce4565b50505050905090810190601f168015611d295780820380516001836020036101000a031916815260200191505b50935050505060405180910390a250505050505050565b60006001600160801b038316611d5857506000611735565b8282026001600160801b038084169080861690831681611d7457fe5b046001600160801b031614611732576040805162461bcd60e51b815260206004820152601f60248201527f75696e74313238206d756c7469706c69636174696f6e206f766572666c6f7700604482015290519081900360640190fd5b600080826001600160801b031611611e2f576040805162461bcd60e51b815260206004820152601860248201527f75696e74313238206469766973696f6e206279207a65726f0000000000000000604482015290519081900360640190fd5b6000826001600160801b0316846001600160801b031681611e4c57fe5b04949350505050565b82805482825590600052602060002090600101600290048101928215611efd5791602002820160005b83821115611ec857835183826101000a8154816001600160801b0302191690836001600160801b031602179055509260200192601001602081600f01049283019260010302611e7e565b8015611efb5782816101000a8154906001600160801b030219169055601001602081600f01049283019260010302611ec8565b505b50611f09929150611f0d565b5090565b5b80821115611f095780546001600160801b0319168155600101611f0e56fe4475726174696f6e73206d75737420626520616464656420696e20617363656e64696e67206f726465724f776e61626c653a206e6577206f776e657220697320746865207a65726f20616464726573734475726174696f6e7320616e642070657263656e7473206d757374206861766520657175616c206c656e67746850657263656e7473206d7573742073756d20746f204d554c5449504c49455220616d6f756e744163636f756e747320616e6420616d6f756e7473206d757374206861766520657175616c206c656e67746850726576696f75732073746172742074696d65206d757374206e6f74206265207265616368656450657263656e742070726f7669646564206d75737420626520736d616c6c6572206f7220657175616c20746f204d554c5449504c4945524f776e61626c653a2063616c6c6572206973206e6f7420746865206f776e6572416c6c6f636174696f6e206d757374206e6f7420626520616c7265616479207265766f6b6564a2646970667358221220b71680e2c8fc98e0b0ff4bdb15e5f404f69af1a90e458131c4cdf1c566ca7fb564736f6c634300060c0033" . parse () . expect ("invalid bytecode")
        });
    pub struct HoprDistributor<M>(ethers::contract::Contract<M>);
    impl<M> Clone for HoprDistributor<M> {
        fn clone(&self) -> Self {
            HoprDistributor(self.0.clone())
        }
    }
    impl<M> std::ops::Deref for HoprDistributor<M> {
        type Target = ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> std::fmt::Debug for HoprDistributor<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(HoprDistributor))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ethers::providers::Middleware> HoprDistributor<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            ethers::contract::Contract::new(address.into(), HOPRDISTRIBUTOR_ABI.clone(), client)
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
                HOPRDISTRIBUTOR_ABI.clone(),
                HOPRDISTRIBUTOR_BYTECODE.clone().into(),
                client,
            );
            let deployer = factory.deploy(constructor_args)?;
            let deployer = ethers::contract::ContractDeployer::new(deployer);
            Ok(deployer)
        }
        #[doc = "Calls the contract's `MULTIPLIER` (0x059f8b16) function"]
        pub fn multiplier(&self) -> ethers::contract::builders::ContractCall<M, u128> {
            self.0
                .method_hash([5, 159, 139, 22], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `addAllocations` (0x22bccfd7) function"]
        pub fn add_allocations(
            &self,
            accounts: ::std::vec::Vec<ethers::core::types::Address>,
            amounts: ::std::vec::Vec<u128>,
            schedule_name: String,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([34, 188, 207, 215], (accounts, amounts, schedule_name))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `addSchedule` (0xe5742970) function"]
        pub fn add_schedule(
            &self,
            durations: ::std::vec::Vec<u128>,
            percents: ::std::vec::Vec<u128>,
            name: String,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([229, 116, 41, 112], (durations, percents, name))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `allocations` (0xc31cd7d7) function"]
        pub fn allocations(
            &self,
            p0: ethers::core::types::Address,
            p1: String,
        ) -> ethers::contract::builders::ContractCall<M, (u128, u128, u128, bool)> {
            self.0
                .method_hash([195, 28, 215, 215], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `claim` (0xf3fe12c9) function"]
        pub fn claim(
            &self,
            schedule_name: String,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([243, 254, 18, 201], schedule_name)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `claimFor` (0xe82330b2) function"]
        pub fn claim_for(
            &self,
            account: ethers::core::types::Address,
            schedule_name: String,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([232, 35, 48, 178], (account, schedule_name))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `getClaimable` (0x70a42898) function"]
        pub fn get_claimable(
            &self,
            account: ethers::core::types::Address,
            schedule_name: String,
        ) -> ethers::contract::builders::ContractCall<M, u128> {
            self.0
                .method_hash([112, 164, 40, 152], (account, schedule_name))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `getSchedule` (0x72840f0e) function"]
        pub fn get_schedule(
            &self,
            name: String,
        ) -> ethers::contract::builders::ContractCall<
            M,
            (::std::vec::Vec<u128>, ::std::vec::Vec<u128>),
        > {
            self.0
                .method_hash([114, 132, 15, 14], name)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `maxMintAmount` (0x239c70ae) function"]
        pub fn max_mint_amount(&self) -> ethers::contract::builders::ContractCall<M, u128> {
            self.0
                .method_hash([35, 156, 112, 174], ())
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
        #[doc = "Calls the contract's `revokeAccount` (0x2c902c7c) function"]
        pub fn revoke_account(
            &self,
            account: ethers::core::types::Address,
            schedule_name: String,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([44, 144, 44, 124], (account, schedule_name))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `startTime` (0x78e97925) function"]
        pub fn start_time(&self) -> ethers::contract::builders::ContractCall<M, u128> {
            self.0
                .method_hash([120, 233, 121, 37], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `token` (0xfc0c546a) function"]
        pub fn token(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Address> {
            self.0
                .method_hash([252, 12, 84, 106], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `totalMinted` (0xa2309ff8) function"]
        pub fn total_minted(&self) -> ethers::contract::builders::ContractCall<M, u128> {
            self.0
                .method_hash([162, 48, 159, 248], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `totalToBeMinted` (0x0373a364) function"]
        pub fn total_to_be_minted(&self) -> ethers::contract::builders::ContractCall<M, u128> {
            self.0
                .method_hash([3, 115, 163, 100], ())
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
        #[doc = "Calls the contract's `updateStartTime` (0xb733f67d) function"]
        pub fn update_start_time(
            &self,
            start_time: u128,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([183, 51, 246, 125], start_time)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Gets the contract's `AllocationAdded` event"]
        pub fn allocation_added_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, AllocationAddedFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `Claimed` event"]
        pub fn claimed_filter(&self) -> ethers::contract::builders::Event<M, ClaimedFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `OwnershipTransferred` event"]
        pub fn ownership_transferred_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, OwnershipTransferredFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `ScheduleAdded` event"]
        pub fn schedule_added_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, ScheduleAddedFilter> {
            self.0.event()
        }
        #[doc = r" Returns an [`Event`](#ethers_contract::builders::Event) builder for all events of this contract"]
        pub fn events(&self) -> ethers::contract::builders::Event<M, HoprDistributorEvents> {
            self.0.event_with_filter(Default::default())
        }
    }
    impl<M: ethers::providers::Middleware> From<ethers::contract::Contract<M>> for HoprDistributor<M> {
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
        name = "AllocationAdded",
        abi = "AllocationAdded(address,uint128,string)"
    )]
    pub struct AllocationAddedFilter {
        #[ethevent(indexed)]
        pub account: ethers::core::types::Address,
        pub amount: u128,
        pub schedule_name: String,
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
    #[ethevent(name = "Claimed", abi = "Claimed(address,uint128,string)")]
    pub struct ClaimedFilter {
        #[ethevent(indexed)]
        pub account: ethers::core::types::Address,
        pub amount: u128,
        pub schedule_name: String,
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
    #[ethevent(
        name = "ScheduleAdded",
        abi = "ScheduleAdded(uint128[],uint128[],string)"
    )]
    pub struct ScheduleAddedFilter {
        pub durations: Vec<u128>,
        pub percents: Vec<u128>,
        pub name: String,
    }
    #[derive(Debug, Clone, PartialEq, Eq, ethers :: contract :: EthAbiType)]
    pub enum HoprDistributorEvents {
        AllocationAddedFilter(AllocationAddedFilter),
        ClaimedFilter(ClaimedFilter),
        OwnershipTransferredFilter(OwnershipTransferredFilter),
        ScheduleAddedFilter(ScheduleAddedFilter),
    }
    impl ethers::contract::EthLogDecode for HoprDistributorEvents {
        fn decode_log(
            log: &ethers::core::abi::RawLog,
        ) -> ::std::result::Result<Self, ethers::core::abi::Error>
        where
            Self: Sized,
        {
            if let Ok(decoded) = AllocationAddedFilter::decode_log(log) {
                return Ok(HoprDistributorEvents::AllocationAddedFilter(decoded));
            }
            if let Ok(decoded) = ClaimedFilter::decode_log(log) {
                return Ok(HoprDistributorEvents::ClaimedFilter(decoded));
            }
            if let Ok(decoded) = OwnershipTransferredFilter::decode_log(log) {
                return Ok(HoprDistributorEvents::OwnershipTransferredFilter(decoded));
            }
            if let Ok(decoded) = ScheduleAddedFilter::decode_log(log) {
                return Ok(HoprDistributorEvents::ScheduleAddedFilter(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData)
        }
    }
    impl ::std::fmt::Display for HoprDistributorEvents {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                HoprDistributorEvents::AllocationAddedFilter(element) => element.fmt(f),
                HoprDistributorEvents::ClaimedFilter(element) => element.fmt(f),
                HoprDistributorEvents::OwnershipTransferredFilter(element) => element.fmt(f),
                HoprDistributorEvents::ScheduleAddedFilter(element) => element.fmt(f),
            }
        }
    }
    #[doc = "Container type for all input parameters for the `MULTIPLIER` function with signature `MULTIPLIER()` and selector `[5, 159, 139, 22]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "MULTIPLIER", abi = "MULTIPLIER()")]
    pub struct MultiplierCall;
    #[doc = "Container type for all input parameters for the `addAllocations` function with signature `addAllocations(address[],uint128[],string)` and selector `[34, 188, 207, 215]`"]
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
        name = "addAllocations",
        abi = "addAllocations(address[],uint128[],string)"
    )]
    pub struct AddAllocationsCall {
        pub accounts: ::std::vec::Vec<ethers::core::types::Address>,
        pub amounts: ::std::vec::Vec<u128>,
        pub schedule_name: String,
    }
    #[doc = "Container type for all input parameters for the `addSchedule` function with signature `addSchedule(uint128[],uint128[],string)` and selector `[229, 116, 41, 112]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "addSchedule", abi = "addSchedule(uint128[],uint128[],string)")]
    pub struct AddScheduleCall {
        pub durations: ::std::vec::Vec<u128>,
        pub percents: ::std::vec::Vec<u128>,
        pub name: String,
    }
    #[doc = "Container type for all input parameters for the `allocations` function with signature `allocations(address,string)` and selector `[195, 28, 215, 215]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "allocations", abi = "allocations(address,string)")]
    pub struct AllocationsCall(pub ethers::core::types::Address, pub String);
    #[doc = "Container type for all input parameters for the `claim` function with signature `claim(string)` and selector `[243, 254, 18, 201]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "claim", abi = "claim(string)")]
    pub struct ClaimCall {
        pub schedule_name: String,
    }
    #[doc = "Container type for all input parameters for the `claimFor` function with signature `claimFor(address,string)` and selector `[232, 35, 48, 178]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "claimFor", abi = "claimFor(address,string)")]
    pub struct ClaimForCall {
        pub account: ethers::core::types::Address,
        pub schedule_name: String,
    }
    #[doc = "Container type for all input parameters for the `getClaimable` function with signature `getClaimable(address,string)` and selector `[112, 164, 40, 152]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "getClaimable", abi = "getClaimable(address,string)")]
    pub struct GetClaimableCall {
        pub account: ethers::core::types::Address,
        pub schedule_name: String,
    }
    #[doc = "Container type for all input parameters for the `getSchedule` function with signature `getSchedule(string)` and selector `[114, 132, 15, 14]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "getSchedule", abi = "getSchedule(string)")]
    pub struct GetScheduleCall {
        pub name: String,
    }
    #[doc = "Container type for all input parameters for the `maxMintAmount` function with signature `maxMintAmount()` and selector `[35, 156, 112, 174]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "maxMintAmount", abi = "maxMintAmount()")]
    pub struct MaxMintAmountCall;
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
    #[doc = "Container type for all input parameters for the `revokeAccount` function with signature `revokeAccount(address,string)` and selector `[44, 144, 44, 124]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "revokeAccount", abi = "revokeAccount(address,string)")]
    pub struct RevokeAccountCall {
        pub account: ethers::core::types::Address,
        pub schedule_name: String,
    }
    #[doc = "Container type for all input parameters for the `startTime` function with signature `startTime()` and selector `[120, 233, 121, 37]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "startTime", abi = "startTime()")]
    pub struct StartTimeCall;
    #[doc = "Container type for all input parameters for the `token` function with signature `token()` and selector `[252, 12, 84, 106]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "token", abi = "token()")]
    pub struct TokenCall;
    #[doc = "Container type for all input parameters for the `totalMinted` function with signature `totalMinted()` and selector `[162, 48, 159, 248]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "totalMinted", abi = "totalMinted()")]
    pub struct TotalMintedCall;
    #[doc = "Container type for all input parameters for the `totalToBeMinted` function with signature `totalToBeMinted()` and selector `[3, 115, 163, 100]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "totalToBeMinted", abi = "totalToBeMinted()")]
    pub struct TotalToBeMintedCall;
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
    #[doc = "Container type for all input parameters for the `updateStartTime` function with signature `updateStartTime(uint128)` and selector `[183, 51, 246, 125]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "updateStartTime", abi = "updateStartTime(uint128)")]
    pub struct UpdateStartTimeCall {
        pub start_time: u128,
    }
    #[derive(Debug, Clone, PartialEq, Eq, ethers :: contract :: EthAbiType)]
    pub enum HoprDistributorCalls {
        Multiplier(MultiplierCall),
        AddAllocations(AddAllocationsCall),
        AddSchedule(AddScheduleCall),
        Allocations(AllocationsCall),
        Claim(ClaimCall),
        ClaimFor(ClaimForCall),
        GetClaimable(GetClaimableCall),
        GetSchedule(GetScheduleCall),
        MaxMintAmount(MaxMintAmountCall),
        Owner(OwnerCall),
        RenounceOwnership(RenounceOwnershipCall),
        RevokeAccount(RevokeAccountCall),
        StartTime(StartTimeCall),
        Token(TokenCall),
        TotalMinted(TotalMintedCall),
        TotalToBeMinted(TotalToBeMintedCall),
        TransferOwnership(TransferOwnershipCall),
        UpdateStartTime(UpdateStartTimeCall),
    }
    impl ethers::core::abi::AbiDecode for HoprDistributorCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::std::result::Result<Self, ethers::core::abi::AbiError> {
            if let Ok(decoded) =
                <MultiplierCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprDistributorCalls::Multiplier(decoded));
            }
            if let Ok(decoded) =
                <AddAllocationsCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprDistributorCalls::AddAllocations(decoded));
            }
            if let Ok(decoded) =
                <AddScheduleCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprDistributorCalls::AddSchedule(decoded));
            }
            if let Ok(decoded) =
                <AllocationsCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprDistributorCalls::Allocations(decoded));
            }
            if let Ok(decoded) = <ClaimCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprDistributorCalls::Claim(decoded));
            }
            if let Ok(decoded) =
                <ClaimForCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprDistributorCalls::ClaimFor(decoded));
            }
            if let Ok(decoded) =
                <GetClaimableCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprDistributorCalls::GetClaimable(decoded));
            }
            if let Ok(decoded) =
                <GetScheduleCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprDistributorCalls::GetSchedule(decoded));
            }
            if let Ok(decoded) =
                <MaxMintAmountCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprDistributorCalls::MaxMintAmount(decoded));
            }
            if let Ok(decoded) = <OwnerCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprDistributorCalls::Owner(decoded));
            }
            if let Ok(decoded) =
                <RenounceOwnershipCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprDistributorCalls::RenounceOwnership(decoded));
            }
            if let Ok(decoded) =
                <RevokeAccountCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprDistributorCalls::RevokeAccount(decoded));
            }
            if let Ok(decoded) =
                <StartTimeCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprDistributorCalls::StartTime(decoded));
            }
            if let Ok(decoded) = <TokenCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprDistributorCalls::Token(decoded));
            }
            if let Ok(decoded) =
                <TotalMintedCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprDistributorCalls::TotalMinted(decoded));
            }
            if let Ok(decoded) =
                <TotalToBeMintedCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprDistributorCalls::TotalToBeMinted(decoded));
            }
            if let Ok(decoded) =
                <TransferOwnershipCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprDistributorCalls::TransferOwnership(decoded));
            }
            if let Ok(decoded) =
                <UpdateStartTimeCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprDistributorCalls::UpdateStartTime(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ethers::core::abi::AbiEncode for HoprDistributorCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                HoprDistributorCalls::Multiplier(element) => element.encode(),
                HoprDistributorCalls::AddAllocations(element) => element.encode(),
                HoprDistributorCalls::AddSchedule(element) => element.encode(),
                HoprDistributorCalls::Allocations(element) => element.encode(),
                HoprDistributorCalls::Claim(element) => element.encode(),
                HoprDistributorCalls::ClaimFor(element) => element.encode(),
                HoprDistributorCalls::GetClaimable(element) => element.encode(),
                HoprDistributorCalls::GetSchedule(element) => element.encode(),
                HoprDistributorCalls::MaxMintAmount(element) => element.encode(),
                HoprDistributorCalls::Owner(element) => element.encode(),
                HoprDistributorCalls::RenounceOwnership(element) => element.encode(),
                HoprDistributorCalls::RevokeAccount(element) => element.encode(),
                HoprDistributorCalls::StartTime(element) => element.encode(),
                HoprDistributorCalls::Token(element) => element.encode(),
                HoprDistributorCalls::TotalMinted(element) => element.encode(),
                HoprDistributorCalls::TotalToBeMinted(element) => element.encode(),
                HoprDistributorCalls::TransferOwnership(element) => element.encode(),
                HoprDistributorCalls::UpdateStartTime(element) => element.encode(),
            }
        }
    }
    impl ::std::fmt::Display for HoprDistributorCalls {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                HoprDistributorCalls::Multiplier(element) => element.fmt(f),
                HoprDistributorCalls::AddAllocations(element) => element.fmt(f),
                HoprDistributorCalls::AddSchedule(element) => element.fmt(f),
                HoprDistributorCalls::Allocations(element) => element.fmt(f),
                HoprDistributorCalls::Claim(element) => element.fmt(f),
                HoprDistributorCalls::ClaimFor(element) => element.fmt(f),
                HoprDistributorCalls::GetClaimable(element) => element.fmt(f),
                HoprDistributorCalls::GetSchedule(element) => element.fmt(f),
                HoprDistributorCalls::MaxMintAmount(element) => element.fmt(f),
                HoprDistributorCalls::Owner(element) => element.fmt(f),
                HoprDistributorCalls::RenounceOwnership(element) => element.fmt(f),
                HoprDistributorCalls::RevokeAccount(element) => element.fmt(f),
                HoprDistributorCalls::StartTime(element) => element.fmt(f),
                HoprDistributorCalls::Token(element) => element.fmt(f),
                HoprDistributorCalls::TotalMinted(element) => element.fmt(f),
                HoprDistributorCalls::TotalToBeMinted(element) => element.fmt(f),
                HoprDistributorCalls::TransferOwnership(element) => element.fmt(f),
                HoprDistributorCalls::UpdateStartTime(element) => element.fmt(f),
            }
        }
    }
    impl ::std::convert::From<MultiplierCall> for HoprDistributorCalls {
        fn from(var: MultiplierCall) -> Self {
            HoprDistributorCalls::Multiplier(var)
        }
    }
    impl ::std::convert::From<AddAllocationsCall> for HoprDistributorCalls {
        fn from(var: AddAllocationsCall) -> Self {
            HoprDistributorCalls::AddAllocations(var)
        }
    }
    impl ::std::convert::From<AddScheduleCall> for HoprDistributorCalls {
        fn from(var: AddScheduleCall) -> Self {
            HoprDistributorCalls::AddSchedule(var)
        }
    }
    impl ::std::convert::From<AllocationsCall> for HoprDistributorCalls {
        fn from(var: AllocationsCall) -> Self {
            HoprDistributorCalls::Allocations(var)
        }
    }
    impl ::std::convert::From<ClaimCall> for HoprDistributorCalls {
        fn from(var: ClaimCall) -> Self {
            HoprDistributorCalls::Claim(var)
        }
    }
    impl ::std::convert::From<ClaimForCall> for HoprDistributorCalls {
        fn from(var: ClaimForCall) -> Self {
            HoprDistributorCalls::ClaimFor(var)
        }
    }
    impl ::std::convert::From<GetClaimableCall> for HoprDistributorCalls {
        fn from(var: GetClaimableCall) -> Self {
            HoprDistributorCalls::GetClaimable(var)
        }
    }
    impl ::std::convert::From<GetScheduleCall> for HoprDistributorCalls {
        fn from(var: GetScheduleCall) -> Self {
            HoprDistributorCalls::GetSchedule(var)
        }
    }
    impl ::std::convert::From<MaxMintAmountCall> for HoprDistributorCalls {
        fn from(var: MaxMintAmountCall) -> Self {
            HoprDistributorCalls::MaxMintAmount(var)
        }
    }
    impl ::std::convert::From<OwnerCall> for HoprDistributorCalls {
        fn from(var: OwnerCall) -> Self {
            HoprDistributorCalls::Owner(var)
        }
    }
    impl ::std::convert::From<RenounceOwnershipCall> for HoprDistributorCalls {
        fn from(var: RenounceOwnershipCall) -> Self {
            HoprDistributorCalls::RenounceOwnership(var)
        }
    }
    impl ::std::convert::From<RevokeAccountCall> for HoprDistributorCalls {
        fn from(var: RevokeAccountCall) -> Self {
            HoprDistributorCalls::RevokeAccount(var)
        }
    }
    impl ::std::convert::From<StartTimeCall> for HoprDistributorCalls {
        fn from(var: StartTimeCall) -> Self {
            HoprDistributorCalls::StartTime(var)
        }
    }
    impl ::std::convert::From<TokenCall> for HoprDistributorCalls {
        fn from(var: TokenCall) -> Self {
            HoprDistributorCalls::Token(var)
        }
    }
    impl ::std::convert::From<TotalMintedCall> for HoprDistributorCalls {
        fn from(var: TotalMintedCall) -> Self {
            HoprDistributorCalls::TotalMinted(var)
        }
    }
    impl ::std::convert::From<TotalToBeMintedCall> for HoprDistributorCalls {
        fn from(var: TotalToBeMintedCall) -> Self {
            HoprDistributorCalls::TotalToBeMinted(var)
        }
    }
    impl ::std::convert::From<TransferOwnershipCall> for HoprDistributorCalls {
        fn from(var: TransferOwnershipCall) -> Self {
            HoprDistributorCalls::TransferOwnership(var)
        }
    }
    impl ::std::convert::From<UpdateStartTimeCall> for HoprDistributorCalls {
        fn from(var: UpdateStartTimeCall) -> Self {
            HoprDistributorCalls::UpdateStartTime(var)
        }
    }
    #[doc = "Container type for all return fields from the `MULTIPLIER` function with signature `MULTIPLIER()` and selector `[5, 159, 139, 22]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct MultiplierReturn(pub u128);
    #[doc = "Container type for all return fields from the `allocations` function with signature `allocations(address,string)` and selector `[195, 28, 215, 215]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct AllocationsReturn {
        pub amount: u128,
        pub claimed: u128,
        pub last_claim: u128,
        pub revoked: bool,
    }
    #[doc = "Container type for all return fields from the `getClaimable` function with signature `getClaimable(address,string)` and selector `[112, 164, 40, 152]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct GetClaimableReturn(pub u128);
    #[doc = "Container type for all return fields from the `getSchedule` function with signature `getSchedule(string)` and selector `[114, 132, 15, 14]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct GetScheduleReturn(pub ::std::vec::Vec<u128>, pub ::std::vec::Vec<u128>);
    #[doc = "Container type for all return fields from the `maxMintAmount` function with signature `maxMintAmount()` and selector `[35, 156, 112, 174]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct MaxMintAmountReturn(pub u128);
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
    #[doc = "Container type for all return fields from the `startTime` function with signature `startTime()` and selector `[120, 233, 121, 37]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct StartTimeReturn(pub u128);
    #[doc = "Container type for all return fields from the `token` function with signature `token()` and selector `[252, 12, 84, 106]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct TokenReturn(pub ethers::core::types::Address);
    #[doc = "Container type for all return fields from the `totalMinted` function with signature `totalMinted()` and selector `[162, 48, 159, 248]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct TotalMintedReturn(pub u128);
    #[doc = "Container type for all return fields from the `totalToBeMinted` function with signature `totalToBeMinted()` and selector `[3, 115, 163, 100]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct TotalToBeMintedReturn(pub u128);
}
