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
            "0x608060405260006001553480156200001657600080fd5b5060405162001fe738038062001fe7833981016040819052620000399162000103565b620000443362000096565b600280546001600160801b03199081166001600160801b0394851617909155600380546001600160a01b0319166001600160a01b0395909516949094179093556004805490931691161790556200015a565b600080546001600160a01b038381166001600160a01b0319831681178455604051919092169283917f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e09190a35050565b80516001600160801b0381168114620000fe57600080fd5b919050565b6000806000606084860312156200011957600080fd5b83516001600160a01b03811681146200013157600080fd5b92506200014160208501620000e6565b91506200015160408501620000e6565b90509250925092565b611e7d806200016a6000396000f3fe608060405234801561001057600080fd5b50600436106101165760003560e01c80638da5cb5b116100a2578063e574297011610071578063e5742970146102c9578063e82330b2146102dc578063f2fde38b146102ef578063f3fe12c914610302578063fc0c546a1461031557600080fd5b80638da5cb5b146101e6578063a2309ff81461020b578063b733f67d1461021e578063c31cd7d71461023157600080fd5b80632c902c7c116100e95780632c902c7c1461018457806370a4289814610197578063715018a6146101aa57806372840f0e146101b257806378e97925146101d357600080fd5b80630373a3641461011b578063059f8b161461015257806322bccfd71461015c578063239c70ae14610171575b600080fd5b60015461013590600160801b90046001600160801b031681565b6040516001600160801b0390911681526020015b60405180910390f35b610135620f424081565b61016f61016a366004611891565b610328565b005b600454610135906001600160801b031681565b61016f610192366004611947565b6106d0565b6101356101a5366004611947565b610859565b61016f6108ce565b6101c56101c036600461199a565b610904565b604051610149929190611a20565b600254610135906001600160801b031681565b6000546001600160a01b03165b6040516001600160a01b039091168152602001610149565b600154610135906001600160801b031681565b61016f61022c366004611a65565b610a5e565b61029461023f366004611a9d565b60066020908152600092835260409092208151808301840180519281529084019290930191909120915280546001909101546001600160801b0380831692600160801b90819004821692918216910460ff1684565b60405161014994939291906001600160801b039485168152928416602084015292166040820152901515606082015260800190565b61016f6102d7366004611891565b610b17565b61016f6102ea366004611947565b610f40565b61016f6102fd366004611b5f565b610f85565b61016f61031036600461199a565b611020565b6003546101f3906001600160a01b031681565b6000546001600160a01b0316331461035b5760405162461bcd60e51b815260040161035290611b7a565b60405180910390fd5b6005828260405161036d929190611baf565b908152604051908190036020019020546000036103c25760405162461bcd60e51b815260206004820152601360248201527214d8da19591d5b19481b5d5cdd08195e1a5cdd606a1b6044820152606401610352565b8483146104255760405162461bcd60e51b815260206004820152602b60248201527f4163636f756e747320616e6420616d6f756e7473206d7573742068617665206560448201526a0e2eac2d840d8cadccee8d60ab1b6064820152608401610352565b600154600160801b90046001600160801b031660005b868110156106aa576006600089898481811061045957610459611bbf565b905060200201602081019061046e9190611b5f565b6001600160a01b03166001600160a01b03168152602001908152602001600020848460405161049e929190611baf565b908152604051908190036020019020546001600160801b0316156105045760405162461bcd60e51b815260206004820152601960248201527f416c6c6f636174696f6e206d757374206e6f74206578697374000000000000006044820152606401610352565b85858281811061051657610516611bbf565b905060200201602081019061052b9190611a65565b600660008a8a8581811061054157610541611bbf565b90506020020160208101906105569190611b5f565b6001600160a01b03166001600160a01b031681526020019081526020016000208585604051610586929190611baf565b90815260405190819003602001902080546001600160801b03929092166001600160801b03199092169190911790556105e5828787848181106105cb576105cb611bbf565b90506020020160208101906105e09190611a65565b611064565b6004549092506001600160801b03908116908316111561060757610607611bd5565b87878281811061061957610619611bbf565b905060200201602081019061062e9190611b5f565b6001600160a01b03167f499c64a5e7cdaa8e72ac9f0f4b080fce7a37c5ef24b2a9f67c7c6a728f5aec0987878481811061066a5761066a611bbf565b905060200201602081019061067f9190611a65565b868660405161069093929190611c14565b60405180910390a2806106a281611c4d565b91505061043b565b50600180546001600160801b03928316600160801b029216919091179055505050505050565b6000546001600160a01b031633146106fa5760405162461bcd60e51b815260040161035290611b7a565b6001600160a01b03831660009081526006602052604080822090516107229085908590611baf565b90815260405190819003602001902080549091506001600160801b03166000036107865760405162461bcd60e51b8152602060048201526015602482015274105b1b1bd8d85d1a5bdb881b5d5cdd08195e1a5cdd605a1b6044820152606401610352565b6001810154600160801b900460ff16156107f15760405162461bcd60e51b815260206004820152602660248201527f416c6c6f636174696f6e206d757374206e6f7420626520616c72656164792072604482015265195d9bdad95960d21b6064820152608401610352565b6001818101805460ff60801b1916600160801b90811790915590548254610836926001600160801b039281900483169261083192808216929004166110de565b6110de565b600180546001600160801b03928316600160801b02921691909117905550505050565b60006108c660058484604051610870929190611baf565b908152602001604051809103902060066000876001600160a01b03166001600160a01b0316815260200190815260200160002085856040516108b3929190611baf565b908152602001604051809103902061114e565b949350505050565b6000546001600160a01b031633146108f85760405162461bcd60e51b815260040161035290611b7a565b6109026000611325565b565b60608060058484604051610919929190611baf565b908152604051908190036020018120906005906109399087908790611baf565b9081526020016040518091039020600101818054806020026020016040519081016040528092919081815260200182805480156109c757602002820191906000526020600020906000905b82829054906101000a90046001600160801b03166001600160801b031681526020019060100190602082600f010492830192600103820291508084116109845790505b5050505050915080805480602002602001604051908101604052809291908181526020018280548015610a4b57602002820191906000526020600020906000905b82829054906101000a90046001600160801b03166001600160801b031681526020019060100190602082600f01049283019260010382029150808411610a085790505b50505050509050915091505b9250929050565b6000546001600160a01b03163314610a885760405162461bcd60e51b815260040161035290611b7a565b6002546001600160801b03428116911611610af55760405162461bcd60e51b815260206004820152602760248201527f50726576696f75732073746172742074696d65206d757374206e6f74206265206044820152661c995858da195960ca1b6064820152608401610352565b600280546001600160801b0319166001600160801b0392909216919091179055565b6000546001600160a01b03163314610b415760405162461bcd60e51b815260040161035290611b7a565b60058282604051610b53929190611baf565b9081526040519081900360200190205415610bb05760405162461bcd60e51b815260206004820152601760248201527f5363686564756c65206d757374206e6f742065786973740000000000000000006044820152606401610352565b848314610c155760405162461bcd60e51b815260206004820152602d60248201527f4475726174696f6e7320616e642070657263656e7473206d757374206861766560448201526c040cae2eac2d840d8cadccee8d609b1b6064820152608401610352565b60008060005b87811015610db857888882818110610c3557610c35611bbf565b9050602002016020810190610c4a9190611a65565b6001600160801b0316836001600160801b031610610cbd5760405162461bcd60e51b815260206004820152602a60248201527f4475726174696f6e73206d75737420626520616464656420696e20617363656e6044820152693234b7339037b93232b960b11b6064820152608401610352565b888882818110610ccf57610ccf611bbf565b9050602002016020810190610ce49190611a65565b9250620f4240878783818110610cfc57610cfc611bbf565b9050602002016020810190610d119190611a65565b6001600160801b03161115610d8e5760405162461bcd60e51b815260206004820152603760248201527f50657263656e742070726f7669646564206d75737420626520736d616c6c657260448201527f206f7220657175616c20746f204d554c5449504c4945520000000000000000006064820152608401610352565b610da4828888848181106105cb576105cb611bbf565b915080610db081611c4d565b915050610c1b565b506001600160801b038116620f424014610e235760405162461bcd60e51b815260206004820152602660248201527f50657263656e7473206d7573742073756d20746f204d554c5449504c49455220604482015265185b5bdd5b9d60d21b6064820152608401610352565b604051806040016040528089898080602002602001604051908101604052809392919081815260200183836020028082843760009201919091525050509082525060408051602089810282810182019093528982529283019290918a918a9182918501908490808284376000920191909152505050915250604051600590610eae9087908790611baf565b90815260200160405180910390206000820151816000019080519060200190610ed892919061173d565b506020828101518051610ef1926001850192019061173d565b509050507f14a9427471da7dbb7ecca56162a326853031d8fab46a74ab7b1b797591d9e468888888888888604051610f2e96959493929190611ca2565b60405180910390a15050505050505050565b610f808383838080601f01602080910402602001604051908101604052809392919081815260200183838082843760009201919091525061137592505050565b505050565b6000546001600160a01b03163314610faf5760405162461bcd60e51b815260040161035290611b7a565b6001600160a01b0381166110145760405162461bcd60e51b815260206004820152602660248201527f4f776e61626c653a206e6577206f776e657220697320746865207a65726f206160448201526564647265737360d01b6064820152608401610352565b61101d81611325565b50565b6110603383838080601f01602080910402602001604051908101604052809392919081815260200183838082843760009201919091525061137592505050565b5050565b6000806110718385611ceb565b9050836001600160801b0316816001600160801b031610156110d55760405162461bcd60e51b815260206004820152601960248201527f75696e74313238206164646974696f6e206f766572666c6f77000000000000006044820152606401610352565b90505b92915050565b6000826001600160801b0316826001600160801b031611156111425760405162461bcd60e51b815260206004820152601c60248201527f75696e74313238207375627472616374696f6e206f766572666c6f77000000006044820152606401610352565b60006108c68385611d16565b60004260025484546001600160801b03928316926111a7921690869060009061117957611179611bbf565b90600052602060002090600291828204019190066010029054906101000a90046001600160801b0316611064565b6001600160801b031611156111be575060006110d8565b4260025484546001600160801b03928316926111f492169086906111e490600190611d3e565b8154811061117957611179611bbf565b6001600160801b03161015611229578154611222906001600160801b0380821691600160801b9004166110de565b90506110d8565b6000805b845481101561131d576002548554600091611260916001600160801b039091169088908590811061117957611179611bbf565b9050426001600160801b0316816001600160801b03161115611282575061131d565b60018501546001600160801b0380831691161061129f575061130b565b84546001870180546113079286926105e0926112fe926001600160801b03169190889081106112d0576112d0611bbf565b90600052602060002090600291828204019190066010029054906101000a90046001600160801b0316611644565b620f42406116d7565b9250505b8061131581611c4d565b91505061122d565b509392505050565b600080546001600160a01b038381166001600160a01b0319831681178455604051919092169283917f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e09190a35050565b6001600160a01b038216600090815260066020526040808220905161139b908490611d85565b90815260405190819003602001902080549091506001600160801b03166114045760405162461bcd60e51b815260206004820152601960248201527f5468657265206973206e6f7468696e6720746f20636c61696d000000000000006044820152606401610352565b6001810154600160801b900460ff16156114555760405162461bcd60e51b81526020600482015260126024820152711058d8dbdd5b9d081a5cc81c995d9bdad95960721b6044820152606401610352565b60006005836040516114679190611d85565b908152602001604051809103902090506000611483828461114e565b83549091506001600160801b0390811690821611156114a4576114a4611bd5565b82546000906114c390600160801b90046001600160801b031683611064565b84549091506001600160801b0390811690821611156114e4576114e4611bd5565b6001546000906114fd906001600160801b031684611064565b6004549091506001600160801b03908116908216111561151f5761151f611bd5565b600180546001600160801b038084166001600160801b0319909216919091179091558554838216600160801b0291161785556115584290565b6001860180546001600160801b0319166001600160801b03928316179055600354604051630dcdc7dd60e41b81526001600160a01b038a8116600483015292861660248201526080604482015260006084820181905260a0606483015260a482015291169063dcdc7dd09060c401600060405180830381600087803b1580156115e057600080fd5b505af11580156115f4573d6000803e3d6000fd5b50505050866001600160a01b03167fd6d52022b5ae5ce877753d56a79a1299605b05220771f26b0817599cabd2b6b48488604051611633929190611da1565b60405180910390a250505050505050565b6000826001600160801b031660000361165f575060006110d8565b600061166b8385611de4565b90506001600160801b0383166116818583611e13565b6001600160801b0316146110d55760405162461bcd60e51b815260206004820152601f60248201527f75696e74313238206d756c7469706c69636174696f6e206f766572666c6f77006044820152606401610352565b600080826001600160801b0316116117315760405162461bcd60e51b815260206004820152601860248201527f75696e74313238206469766973696f6e206279207a65726f00000000000000006044820152606401610352565b60006108c68385611e13565b828054828255906000526020600020906001016002900481019282156117e55791602002820160005b838211156117b057835183826101000a8154816001600160801b0302191690836001600160801b031602179055509260200192601001602081600f01049283019260010302611766565b80156117e35782816101000a8154906001600160801b030219169055601001602081600f010492830192600103026117b0565b505b506117f19291506117f5565b5090565b5b808211156117f157600081556001016117f6565b60008083601f84011261181c57600080fd5b50813567ffffffffffffffff81111561183457600080fd5b6020830191508360208260051b8501011115610a5757600080fd5b60008083601f84011261186157600080fd5b50813567ffffffffffffffff81111561187957600080fd5b602083019150836020828501011115610a5757600080fd5b600080600080600080606087890312156118aa57600080fd5b863567ffffffffffffffff808211156118c257600080fd5b6118ce8a838b0161180a565b909850965060208901359150808211156118e757600080fd5b6118f38a838b0161180a565b9096509450604089013591508082111561190c57600080fd5b5061191989828a0161184f565b979a9699509497509295939492505050565b80356001600160a01b038116811461194257600080fd5b919050565b60008060006040848603121561195c57600080fd5b6119658461192b565b9250602084013567ffffffffffffffff81111561198157600080fd5b61198d8682870161184f565b9497909650939450505050565b600080602083850312156119ad57600080fd5b823567ffffffffffffffff8111156119c457600080fd5b6119d08582860161184f565b90969095509350505050565b600081518084526020808501945080840160005b83811015611a155781516001600160801b0316875295820195908201906001016119f0565b509495945050505050565b604081526000611a3360408301856119dc565b8281036020840152611a4581856119dc565b95945050505050565b80356001600160801b038116811461194257600080fd5b600060208284031215611a7757600080fd5b611a8082611a4e565b9392505050565b634e487b7160e01b600052604160045260246000fd5b60008060408385031215611ab057600080fd5b611ab98361192b565b9150602083013567ffffffffffffffff80821115611ad657600080fd5b818501915085601f830112611aea57600080fd5b813581811115611afc57611afc611a87565b604051601f8201601f19908116603f01168101908382118183101715611b2457611b24611a87565b81604052828152886020848701011115611b3d57600080fd5b8260208601602083013760006020848301015280955050505050509250929050565b600060208284031215611b7157600080fd5b611a808261192b565b6020808252818101527f4f776e61626c653a2063616c6c6572206973206e6f7420746865206f776e6572604082015260600190565b8183823760009101908152919050565b634e487b7160e01b600052603260045260246000fd5b634e487b7160e01b600052600160045260246000fd5b81835281816020850137506000828201602090810191909152601f909101601f19169091010190565b6001600160801b0384168152604060208201526000611a45604083018486611beb565b634e487b7160e01b600052601160045260246000fd5b600060018201611c5f57611c5f611c37565b5060010190565b8183526000602080850194508260005b85811015611a15576001600160801b03611c8f83611a4e565b1687529582019590820190600101611c76565b606081526000611cb660608301888a611c66565b8281036020840152611cc9818789611c66565b90508281036040840152611cde818587611beb565b9998505050505050505050565b60006001600160801b03808316818516808303821115611d0d57611d0d611c37565b01949350505050565b60006001600160801b0383811690831681811015611d3657611d36611c37565b039392505050565b600082821015611d5057611d50611c37565b500390565b60005b83811015611d70578181015183820152602001611d58565b83811115611d7f576000848401525b50505050565b60008251611d97818460208701611d55565b9190910192915050565b6001600160801b03831681526040602082015260008251806040840152611dcf816060850160208701611d55565b601f01601f1916919091016060019392505050565b60006001600160801b0380831681851681830481118215151615611e0a57611e0a611c37565b02949350505050565b60006001600160801b0380841680611e3b57634e487b7160e01b600052601260045260246000fd5b9216919091049291505056fea2646970667358221220b824264250991f59afb731dc5eb4dab6a0905aa8b22cad6dfae37ebac5bade4864736f6c634300080d0033" . parse () . expect ("invalid bytecode")
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
