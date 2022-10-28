pub use tickets_utils::*;
#[allow(clippy::too_many_arguments, non_camel_case_types)]
pub mod tickets_utils {
    #![allow(clippy::enum_variant_names)]
    #![allow(dead_code)]
    #![allow(clippy::type_complexity)]
    #![allow(unused_imports)]
    pub use super::super::shared_types::*;
    use ethers::contract::{
        builders::{ContractCall, Event},
        Contract, Lazy,
    };
    use ethers::core::{
        abi::{Abi, Detokenize, InvalidOutputType, Token, Tokenizable},
        types::*,
    };
    use ethers::providers::Middleware;
    #[doc = "TicketsUtils was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    # [rustfmt :: skip] const __ABI : & str = "[{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"bytes\",\"name\":\"publicKey\",\"type\":\"bytes\",\"components\":[],\"indexed\":false},{\"internalType\":\"bytes\",\"name\":\"multiaddr\",\"type\":\"bytes\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"Announcement\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"source\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"destination\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"bytes32\",\"name\":\"newCommitment\",\"type\":\"bytes32\",\"components\":[],\"indexed\":false},{\"internalType\":\"uint256\",\"name\":\"ticketEpoch\",\"type\":\"uint256\",\"components\":[],\"indexed\":false},{\"internalType\":\"uint256\",\"name\":\"channelBalance\",\"type\":\"uint256\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"ChannelBumped\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"funder\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"source\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"destination\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"ChannelFunded\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"source\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"destination\",\"type\":\"address\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"ChannelOpened\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"source\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"destination\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"struct HoprChannels.Channel\",\"name\":\"newState\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint256\",\"name\":\"balance\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"commitment\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"ticketEpoch\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"ticketIndex\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"enum HoprChannels.ChannelStatus\",\"name\":\"status\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"channelEpoch\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint32\",\"name\":\"closureTime\",\"type\":\"uint32\",\"components\":[]}],\"indexed\":false}],\"type\":\"event\",\"name\":\"ChannelUpdated\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"source\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"destination\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"bytes32\",\"name\":\"nextCommitment\",\"type\":\"bytes32\",\"components\":[],\"indexed\":false},{\"internalType\":\"uint256\",\"name\":\"ticketEpoch\",\"type\":\"uint256\",\"components\":[],\"indexed\":false},{\"internalType\":\"uint256\",\"name\":\"ticketIndex\",\"type\":\"uint256\",\"components\":[],\"indexed\":false},{\"internalType\":\"bytes32\",\"name\":\"proofOfRelaySecret\",\"type\":\"bytes32\",\"components\":[],\"indexed\":false},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[],\"indexed\":false},{\"internalType\":\"uint256\",\"name\":\"winProb\",\"type\":\"uint256\",\"components\":[],\"indexed\":false},{\"internalType\":\"bytes\",\"name\":\"signature\",\"type\":\"bytes\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"TicketRedeemed\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"log\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"log_address\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"uint256[]\",\"name\":\"val\",\"type\":\"uint256[]\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"log_array\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"int256[]\",\"name\":\"val\",\"type\":\"int256[]\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"log_array\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address[]\",\"name\":\"val\",\"type\":\"address[]\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"log_array\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"bytes\",\"name\":\"\",\"type\":\"bytes\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"log_bytes\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"log_bytes32\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"int256\",\"name\":\"\",\"type\":\"int256\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"log_int\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"key\",\"type\":\"string\",\"components\":[],\"indexed\":false},{\"internalType\":\"address\",\"name\":\"val\",\"type\":\"address\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"log_named_address\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"key\",\"type\":\"string\",\"components\":[],\"indexed\":false},{\"internalType\":\"uint256[]\",\"name\":\"val\",\"type\":\"uint256[]\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"log_named_array\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"key\",\"type\":\"string\",\"components\":[],\"indexed\":false},{\"internalType\":\"int256[]\",\"name\":\"val\",\"type\":\"int256[]\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"log_named_array\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"key\",\"type\":\"string\",\"components\":[],\"indexed\":false},{\"internalType\":\"address[]\",\"name\":\"val\",\"type\":\"address[]\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"log_named_array\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"key\",\"type\":\"string\",\"components\":[],\"indexed\":false},{\"internalType\":\"bytes\",\"name\":\"val\",\"type\":\"bytes\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"log_named_bytes\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"key\",\"type\":\"string\",\"components\":[],\"indexed\":false},{\"internalType\":\"bytes32\",\"name\":\"val\",\"type\":\"bytes32\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"log_named_bytes32\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"key\",\"type\":\"string\",\"components\":[],\"indexed\":false},{\"internalType\":\"int256\",\"name\":\"val\",\"type\":\"int256\",\"components\":[],\"indexed\":false},{\"internalType\":\"uint256\",\"name\":\"decimals\",\"type\":\"uint256\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"log_named_decimal_int\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"key\",\"type\":\"string\",\"components\":[],\"indexed\":false},{\"internalType\":\"uint256\",\"name\":\"val\",\"type\":\"uint256\",\"components\":[],\"indexed\":false},{\"internalType\":\"uint256\",\"name\":\"decimals\",\"type\":\"uint256\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"log_named_decimal_uint\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"key\",\"type\":\"string\",\"components\":[],\"indexed\":false},{\"internalType\":\"int256\",\"name\":\"val\",\"type\":\"int256\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"log_named_int\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"key\",\"type\":\"string\",\"components\":[],\"indexed\":false},{\"internalType\":\"string\",\"name\":\"val\",\"type\":\"string\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"log_named_string\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"key\",\"type\":\"string\",\"components\":[],\"indexed\":false},{\"internalType\":\"uint256\",\"name\":\"val\",\"type\":\"uint256\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"log_named_uint\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"log_string\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"log_uint\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"bytes\",\"name\":\"\",\"type\":\"bytes\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"logs\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"IS_SCRIPT\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"IS_TEST\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"PROOF_OF_RELAY_SECRET_0\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"SECRET_0\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"SECRET_1\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"SECRET_2\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"TICKET_AB_LOSS\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"source\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"nextCommitment\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"ticketEpoch\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"ticketIndex\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"proofOfRelaySecret\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"winProb\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"signature\",\"type\":\"bytes\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"TICKET_AB_WIN\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"source\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"nextCommitment\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"ticketEpoch\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"ticketIndex\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"proofOfRelaySecret\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"winProb\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"signature\",\"type\":\"bytes\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"TICKET_AB_WIN_RECYCLED\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"source\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"nextCommitment\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"ticketEpoch\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"ticketIndex\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"proofOfRelaySecret\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"winProb\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"signature\",\"type\":\"bytes\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"TICKET_BA_WIN\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"source\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"nextCommitment\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"ticketEpoch\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"ticketIndex\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"proofOfRelaySecret\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"winProb\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"signature\",\"type\":\"bytes\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"WIN_PROB_0\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"WIN_PROB_100\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"struct HoprChannels.Channel\",\"name\":\"channel1\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint256\",\"name\":\"balance\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"commitment\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"ticketEpoch\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"ticketIndex\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"enum HoprChannels.ChannelStatus\",\"name\":\"status\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"channelEpoch\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint32\",\"name\":\"closureTime\",\"type\":\"uint32\",\"components\":[]}]},{\"internalType\":\"struct HoprChannels.Channel\",\"name\":\"channel2\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint256\",\"name\":\"balance\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"commitment\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"ticketEpoch\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"ticketIndex\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"enum HoprChannels.ChannelStatus\",\"name\":\"status\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"channelEpoch\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint32\",\"name\":\"closureTime\",\"type\":\"uint32\",\"components\":[]}]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"assertEqChannels\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"failed\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"contract HoprChannels\",\"name\":\"hoprChannels\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"channelId\",\"type\":\"bytes32\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"getChannelFromTuple\",\"outputs\":[{\"internalType\":\"struct HoprChannels.Channel\",\"name\":\"\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint256\",\"name\":\"balance\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"commitment\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"ticketEpoch\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"ticketIndex\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"enum HoprChannels.ChannelStatus\",\"name\":\"status\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"channelEpoch\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint32\",\"name\":\"closureTime\",\"type\":\"uint32\",\"components\":[]}]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"source\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"destination\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"pure\",\"type\":\"function\",\"name\":\"getChannelId\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"vm\",\"outputs\":[{\"internalType\":\"contract Vm\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]}]" ;
    #[doc = r" The parsed JSON-ABI of the contract."]
    pub static TICKETSUTILS_ABI: ethers::contract::Lazy<ethers::core::abi::Abi> =
        ethers::contract::Lazy::new(|| {
            ethers::core::utils::__serde_json::from_str(__ABI).expect("invalid abi")
        });
    #[doc = r" Bytecode of the #name contract"]
    pub static TICKETSUTILS_BYTECODE: ethers::contract::Lazy<ethers::core::types::Bytes> =
        ethers::contract::Lazy::new(|| {
            "0x6000805462ff00ff19166201000117905573f39fd6e51aad88f6f4ce6ab8827279cfffb9226660809081527fac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff8060a052610140604090815260e081815260c091620015fe6101003990528051600880546001600160a01b0319166001600160a01b0390921691909117815560208083015160095560408301518051620000a992600a9201906200066f565b50505060405180606001604052807370997970c51812dc3a010c7d01b50e0d17dc79c86001600160a01b031681526020017f59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d8152602001604051806060016040528060408152602001620014fe6040913990528051600b80546001600160a01b0319166001600160a01b03909216919091178155602080830151600c55604083015180516200015d92600d9201906200066f565b5050506040518060600160405280733c44cdddb6a900fa2b585dd299e03d12fa4293bc6001600160a01b031681526020017f5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a81526020016040518060600160405280604081526020016200163e6040913990528051600e80546001600160a01b0319166001600160a01b03909216919091178155602080830151600f5560408301518051620002119260109201906200066f565b5050604051651cd958dc995d60d21b6020820152602601905060408051601f1981840301815282825280516020918201206011819055908301520160408051601f1981840301815282825280516020918201206012819055908301520160408051601f198184030181529082905280516020918201206013557f50524f4f465f4f465f52454c41595f5345435245545f300000000000000000009082015260370160408051601f1981840301815282825280516020918201206014819055600019601581905560006016819055610100860185526008546001600160a01b03168652601254868501528585015260016060808701919091526080860192909252600a60a086015260c08501528251908101835282815260e084019290916200157e9083013990528051601780546001600160a01b0319166001600160a01b0390921691909117815560208083015160185560408301516019556060830151601a556080830151601b5560a0830151601c5560c0830151601d5560e08301518051620003a192601e9201906200066f565b505050604051806101000160405280600860000160009054906101000a90046001600160a01b03166001600160a01b03168152602001601254815260200160008152602001600181526020016014548152602001600a81526020016015548152602001604051806060016040528060408152602001620014be6040913990528051601f80546001600160a01b0319166001600160a01b03909216919091178155602080830151815560408301516021556060830151602255608083015160235560a083015160245560c083015160255560e08301518051620004889260269201906200066f565b505050604051806101000160405280600b60000160009054906101000a90046001600160a01b03166001600160a01b03168152602001601254815260200160008152602001600181526020016014548152602001600a815260200160165481526020016040518060600160405280604081526020016200153e6040913990528051602780546001600160a01b0319166001600160a01b0390921691909117815560208083015160285560408301516029556060830151602a556080830151602b5560a0830151602c5560c0830151602d5560e083015180516200057092602e9201906200066f565b505050604051806101000160405280600b60000160009054906101000a90046001600160a01b03166001600160a01b03168152602001601254815260200160008152602001600181526020016014548152602001600a81526020016015548152602001604051806060016040528060408152602001620015be6040913990528051602f80546001600160a01b0319166001600160a01b0390921691909117815560208083015160305560408301516031556060830151603255608083015160335560a083015160345560c083015160355560e08301518051620006589260369201906200066f565b5050503480156200066857600080fd5b5062000751565b8280546200067d9062000715565b90600052602060002090601f016020900481019282620006a15760008555620006ec565b82601f10620006bc57805160ff1916838001178555620006ec565b82800160010185558215620006ec579182015b82811115620006ec578251825591602001919060010190620006cf565b50620006fa929150620006fe565b5090565b5b80821115620006fa5760008155600101620006ff565b600181811c908216806200072a57607f821691505b6020821081036200074b57634e487b7160e01b600052602260045260246000fd5b50919050565b610d5d80620007616000396000f3fe608060405234801561001057600080fd5b506004361061010b5760003560e01c8063939d09d9116100a2578063ba414fa611610071578063ba414fa61461024a578063cfffcb8c14610252578063d76a1a531461025b578063f8ccbf4714610263578063fa7626d41461027657600080fd5b8063939d09d9146101fe5780639c946fa61461021a5780639f91ffaf14610222578063b013655a1461022a57600080fd5b80636901e571116100de5780636901e5711461017157806379d8d6d91461017a5780637aa56adc146101835780637b0aed3d146101a657600080fd5b80633a7684631461011057806343e93acb1461014857806356369cba1461015f5780635e93925514610168575b600080fd5b61012b737109709ecfa91a80626ff3989d68f67f5b1dd12d81565b6040516001600160a01b0390911681526020015b60405180910390f35b61015160145481565b60405190815260200161013f565b61015160165481565b61015160125481565b61015160135481565b61015160155481565b6101966101913660046109cc565b610283565b604051901515815260200161013f565b6101516101b4366004610a17565b6040516bffffffffffffffffffffffff19606084811b8216602084015283901b16603482015260009060480160405160208183030381529060405280519060200120905092915050565b6102066102b0565b60405161013f989796959493929190610a80565b61020661036f565b6102066103ab565b61023d610238366004610af1565b6103e7565b60405161013f9190610b33565b610196610501565b61015160115481565b61020661062c565b6000546101969062010000900460ff1681565b6000546101969060ff1681565b60008061028f84610668565b9050600061029c84610668565b90506102a882826106ba565b505092915050565b60278054602854602954602a54602b54602c54602d54602e80546001600160a01b0390981698969795969495939492939192916102ec90610ba7565b80601f016020809104026020016040519081016040528092919081815260200182805461031890610ba7565b80156103655780601f1061033a57610100808354040283529160200191610365565b820191906000526020600020905b81548152906001019060200180831161034857829003601f168201915b5050505050905088565b602f8054603054603154603254603354603454603554603680546001600160a01b0390981698969795969495939492939192916102ec90610ba7565b60178054601854601954601a54601b54601c54601d54601e80546001600160a01b0390981698969795969495939492939192916102ec90610ba7565b6040805160e081018252600080825260208201819052918101829052606081018290526080810182905260a0810182905260c08101919091526000806000806000806000896001600160a01b0316637a7ebd7b8a6040518263ffffffff1660e01b815260040161045991815260200190565b60e060405180830381865afa158015610476573d6000803e3d6000fd5b505050506040513d601f19601f8201168201806040525081019061049a9190610be1565b96509650965096509650965096506040518060e001604052808881526020018781526020018681526020018581526020018460038111156104dd576104dd610b1d565b81526020018381526020018263ffffffff1681525097505050505050505092915050565b60008054610100900460ff16156105215750600054610100900460ff1690565b6000737109709ecfa91a80626ff3989d68f67f5b1dd12d3b156106275760408051737109709ecfa91a80626ff3989d68f67f5b1dd12d602082018190526519985a5b195960d21b828401528251808303840181526060830190935260009290916105af917f667f9d70ca411d70ead50d8d5c22070dafc36ad75f3dcf5e7237b22ade9aecc491608001610c4b565b60408051601f19818403018152908290526105c991610c7c565b6000604051808303816000865af19150503d8060008114610606576040519150601f19603f3d011682016040523d82523d6000602084013e61060b565b606091505b50915050808060200190518101906106239190610c98565b9150505b919050565b601f8054602054602154602254602354602454602554602680546001600160a01b0390981698969795969495939492939192916102ec90610ba7565b80516020808301516040808501516060860151608087015160a088015160c0890151945160009861069d989097969101610cc1565b604051602081830303815290604052805190602001209050919050565b8082146107e4577f41304facd9323d75b11bcdd609cb38effffdb05710f7caf0e9b16c6d9d709f5060405161072e9060208082526025908201527f4572726f723a2061203d3d2062206e6f7420736174697366696564205b627974604082015264657333325d60d81b606082015260800190565b60405180910390a160408051818152600a81830152690808115e1c1958dd195960b21b60608201526020810183905290517fafb795c9c61e4fe7468c386f925d7a5429ecad9c0495ddb8d38d690614d32f999181900360800190a160408051818152600a8183015269080808081058dd1d585b60b21b60608201526020810184905290517fafb795c9c61e4fe7468c386f925d7a5429ecad9c0495ddb8d38d690614d32f999181900360800190a16107e46107e8565b5050565b737109709ecfa91a80626ff3989d68f67f5b1dd12d3b156108e35760408051737109709ecfa91a80626ff3989d68f67f5b1dd12d602082018190526519985a5b195960d21b9282019290925260016060820152600091907f70ca10bbd0dbfd9020a9f4b13402c16cb120705e0d1c0aeab10fa353ae586fc49060800160408051601f19818403018152908290526108829291602001610c4b565b60408051601f198184030181529082905261089c91610c7c565b6000604051808303816000865af19150503d80600081146108d9576040519150601f19603f3d011682016040523d82523d6000602084013e6108de565b606091505b505050505b6000805461ff001916610100179055565b6004811061090157600080fd5b50565b8035610627816108f4565b63ffffffff8116811461090157600080fd5b80356106278161090f565b600060e0828403121561093e57600080fd5b60405160e0810181811067ffffffffffffffff8211171561096f57634e487b7160e01b600052604160045260246000fd5b8060405250809150823581526020830135602082015260408301356040820152606083013560608201526109a560808401610904565b608082015260a083013560a08201526109c060c08401610921565b60c08201525092915050565b6000806101c083850312156109e057600080fd5b6109ea848461092c565b91506109f98460e0850161092c565b90509250929050565b6001600160a01b038116811461090157600080fd5b60008060408385031215610a2a57600080fd5b8235610a3581610a02565b91506020830135610a4581610a02565b809150509250929050565b60005b83811015610a6b578181015183820152602001610a53565b83811115610a7a576000848401525b50505050565b600061010060018060a01b038b1683528960208401528860408401528760608401528660808401528560a08401528460c08401528060e0840152835180828501526101209150610ad68183860160208801610a50565b601f01601f1916929092019091019998505050505050505050565b60008060408385031215610b0457600080fd5b8235610b0f81610a02565b946020939093013593505050565b634e487b7160e01b600052602160045260246000fd5b600060e08201905082518252602083015160208301526040830151604083015260608301516060830152608083015160048110610b8057634e487b7160e01b600052602160045260246000fd5b8060808401525060a083015160a083015263ffffffff60c08401511660c083015292915050565b600181811c90821680610bbb57607f821691505b602082108103610bdb57634e487b7160e01b600052602260045260246000fd5b50919050565b600080600080600080600060e0888a031215610bfc57600080fd5b875196506020880151955060408801519450606088015193506080880151610c23816108f4565b60a089015160c08a01519194509250610c3b8161090f565b8091505092959891949750929550565b6001600160e01b0319831681528151600090610c6e816004850160208701610a50565b919091016004019392505050565b60008251610c8e818460208701610a50565b9190910192915050565b600060208284031215610caa57600080fd5b81518015158114610cba57600080fd5b9392505050565b878152866020820152856040820152846060820152600060048510610cf657634e487b7160e01b600052602160045260246000fd5b5060f89390931b6080840152608183019190915260e01b6001600160e01b03191660a182015260a50194935050505056fea2646970667358221220619370cf489a4b2a916991b72d86d5d0645538bd273d02df5f842d19be758a0064736f6c634300080d003340e302cb0b8b18dbdd08ca1bfc93f1f2c40d5b93e8366dbd97f323aabb26e05f91b2e8828a3a15ffb39d10c55e3fbc2fca72c4d3a3083f09fe07797a6a9ecc54ba5734d8f7091719471e7f7ed6b9df170dc70cc661ca05e688601ad984f068b0d67351e5f06073092499336ab0839ef8a521afd334e53807205fa2f08eec74f4c81d8a3fe9d2dfbbf916bad5c3ff2acfb557c4972eb172f6441b85058e8cbd26b67afed7d5e72eae4e5bbf0b4ed9d949c0b06b0755b81b80742e4898f36fcc33e28514db6bf62eab85e382e77a551639f616a51e527480dc922004a197b446006e396a671f35c69bbe966fbf26ccdd31da7722ea566fd34ba2724f6a10d7231b43bbf7f4a28786e47be61b6e3c40f4ff95f214e0ac3b43b10d9d962a076e7e0f0a35e4a487ba460af46f9b061e3474c1af399a50033a3f6a48f84a279acdc9818318535b54105d4a7aae60c08fc45f9687181b4fdfc625bd1a753fa7397fed753547f11ca8696646f2f3acb08e31016afac23e630c5d11f59f61fef57b0d2aa59d9031e97dd78ff8c15aa86939de9b1e791066a0224e331bc962a2099a7b1f0464b8bbafe1535f2301c72c2cb3535b172da30b02686ab0393d348614f157fbdb" . parse () . expect ("invalid bytecode")
        });
    pub struct TicketsUtils<M>(ethers::contract::Contract<M>);
    impl<M> Clone for TicketsUtils<M> {
        fn clone(&self) -> Self {
            TicketsUtils(self.0.clone())
        }
    }
    impl<M> std::ops::Deref for TicketsUtils<M> {
        type Target = ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> std::fmt::Debug for TicketsUtils<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(TicketsUtils))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ethers::providers::Middleware> TicketsUtils<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            ethers::contract::Contract::new(address.into(), TICKETSUTILS_ABI.clone(), client).into()
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
                TICKETSUTILS_ABI.clone(),
                TICKETSUTILS_BYTECODE.clone().into(),
                client,
            );
            let deployer = factory.deploy(constructor_args)?;
            let deployer = ethers::contract::ContractDeployer::new(deployer);
            Ok(deployer)
        }
        #[doc = "Calls the contract's `IS_SCRIPT` (0xf8ccbf47) function"]
        pub fn is_script(&self) -> ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([248, 204, 191, 71], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `IS_TEST` (0xfa7626d4) function"]
        pub fn is_test(&self) -> ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([250, 118, 38, 212], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `PROOF_OF_RELAY_SECRET_0` (0x43e93acb) function"]
        pub fn proof_of_relay_secret_0(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([67, 233, 58, 203], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `SECRET_0` (0xcfffcb8c) function"]
        pub fn secret_0(&self) -> ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([207, 255, 203, 140], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `SECRET_1` (0x5e939255) function"]
        pub fn secret_1(&self) -> ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([94, 147, 146, 85], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `SECRET_2` (0x6901e571) function"]
        pub fn secret_2(&self) -> ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([105, 1, 229, 113], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `TICKET_AB_LOSS` (0x939d09d9) function"]
        pub fn ticket_ab_loss(
            &self,
        ) -> ethers::contract::builders::ContractCall<
            M,
            (
                ethers::core::types::Address,
                [u8; 32],
                ethers::core::types::U256,
                ethers::core::types::U256,
                [u8; 32],
                ethers::core::types::U256,
                ethers::core::types::U256,
                ethers::core::types::Bytes,
            ),
        > {
            self.0
                .method_hash([147, 157, 9, 217], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `TICKET_AB_WIN` (0x9f91ffaf) function"]
        pub fn ticket_ab_win(
            &self,
        ) -> ethers::contract::builders::ContractCall<
            M,
            (
                ethers::core::types::Address,
                [u8; 32],
                ethers::core::types::U256,
                ethers::core::types::U256,
                [u8; 32],
                ethers::core::types::U256,
                ethers::core::types::U256,
                ethers::core::types::Bytes,
            ),
        > {
            self.0
                .method_hash([159, 145, 255, 175], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `TICKET_AB_WIN_RECYCLED` (0x9c946fa6) function"]
        pub fn ticket_ab_win_recycled(
            &self,
        ) -> ethers::contract::builders::ContractCall<
            M,
            (
                ethers::core::types::Address,
                [u8; 32],
                ethers::core::types::U256,
                ethers::core::types::U256,
                [u8; 32],
                ethers::core::types::U256,
                ethers::core::types::U256,
                ethers::core::types::Bytes,
            ),
        > {
            self.0
                .method_hash([156, 148, 111, 166], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `TICKET_BA_WIN` (0xd76a1a53) function"]
        pub fn ticket_ba_win(
            &self,
        ) -> ethers::contract::builders::ContractCall<
            M,
            (
                ethers::core::types::Address,
                [u8; 32],
                ethers::core::types::U256,
                ethers::core::types::U256,
                [u8; 32],
                ethers::core::types::U256,
                ethers::core::types::U256,
                ethers::core::types::Bytes,
            ),
        > {
            self.0
                .method_hash([215, 106, 26, 83], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `WIN_PROB_0` (0x56369cba) function"]
        pub fn win_prob_0(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([86, 54, 156, 186], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `WIN_PROB_100` (0x79d8d6d9) function"]
        pub fn win_prob_100(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([121, 216, 214, 217], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `assertEqChannels` (0x7aa56adc) function"]
        pub fn assert_eq_channels(
            &self,
            channel_1: Channel,
            channel_2: Channel,
        ) -> ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([122, 165, 106, 220], (channel_1, channel_2))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `failed` (0xba414fa6) function"]
        pub fn failed(&self) -> ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([186, 65, 79, 166], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `getChannelFromTuple` (0xb013655a) function"]
        pub fn get_channel_from_tuple(
            &self,
            hopr_channels: ethers::core::types::Address,
            channel_id: [u8; 32],
        ) -> ethers::contract::builders::ContractCall<M, Channel> {
            self.0
                .method_hash([176, 19, 101, 90], (hopr_channels, channel_id))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `getChannelId` (0x7b0aed3d) function"]
        pub fn get_channel_id(
            &self,
            source: ethers::core::types::Address,
            destination: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([123, 10, 237, 61], (source, destination))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `vm` (0x3a768463) function"]
        pub fn vm(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Address> {
            self.0
                .method_hash([58, 118, 132, 99], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Gets the contract's `Announcement` event"]
        pub fn announcement_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, AnnouncementFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `ChannelBumped` event"]
        pub fn channel_bumped_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, ChannelBumpedFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `ChannelFunded` event"]
        pub fn channel_funded_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, ChannelFundedFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `ChannelOpened` event"]
        pub fn channel_opened_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, ChannelOpenedFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `ChannelUpdated` event"]
        pub fn channel_updated_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, ChannelUpdatedFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `TicketRedeemed` event"]
        pub fn ticket_redeemed_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, TicketRedeemedFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `log` event"]
        pub fn log_filter(&self) -> ethers::contract::builders::Event<M, LogFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `log_address` event"]
        pub fn log_address_filter(&self) -> ethers::contract::builders::Event<M, LogAddressFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `log_array` event"]
        pub fn log_array_1_filter(&self) -> ethers::contract::builders::Event<M, LogArray1Filter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `log_array` event"]
        pub fn log_array_2_filter(&self) -> ethers::contract::builders::Event<M, LogArray2Filter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `log_array` event"]
        pub fn log_array_3_filter(&self) -> ethers::contract::builders::Event<M, LogArray3Filter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `log_bytes` event"]
        pub fn log_bytes_filter(&self) -> ethers::contract::builders::Event<M, LogBytesFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `log_bytes32` event"]
        pub fn log_bytes_32_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, LogBytes32Filter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `log_int` event"]
        pub fn log_int_filter(&self) -> ethers::contract::builders::Event<M, LogIntFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `log_named_address` event"]
        pub fn log_named_address_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, LogNamedAddressFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `log_named_array` event"]
        pub fn log_named_array_1_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, LogNamedArray1Filter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `log_named_array` event"]
        pub fn log_named_array_2_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, LogNamedArray2Filter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `log_named_array` event"]
        pub fn log_named_array_3_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, LogNamedArray3Filter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `log_named_bytes` event"]
        pub fn log_named_bytes_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, LogNamedBytesFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `log_named_bytes32` event"]
        pub fn log_named_bytes_32_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, LogNamedBytes32Filter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `log_named_decimal_int` event"]
        pub fn log_named_decimal_int_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, LogNamedDecimalIntFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `log_named_decimal_uint` event"]
        pub fn log_named_decimal_uint_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, LogNamedDecimalUintFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `log_named_int` event"]
        pub fn log_named_int_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, LogNamedIntFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `log_named_string` event"]
        pub fn log_named_string_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, LogNamedStringFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `log_named_uint` event"]
        pub fn log_named_uint_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, LogNamedUintFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `log_string` event"]
        pub fn log_string_filter(&self) -> ethers::contract::builders::Event<M, LogStringFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `log_uint` event"]
        pub fn log_uint_filter(&self) -> ethers::contract::builders::Event<M, LogUintFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `logs` event"]
        pub fn logs_filter(&self) -> ethers::contract::builders::Event<M, LogsFilter> {
            self.0.event()
        }
        #[doc = r" Returns an [`Event`](#ethers_contract::builders::Event) builder for all events of this contract"]
        pub fn events(&self) -> ethers::contract::builders::Event<M, TicketsUtilsEvents> {
            self.0.event_with_filter(Default::default())
        }
    }
    impl<M: ethers::providers::Middleware> From<ethers::contract::Contract<M>> for TicketsUtils<M> {
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
    #[ethevent(name = "Announcement", abi = "Announcement(address,bytes,bytes)")]
    pub struct AnnouncementFilter {
        #[ethevent(indexed)]
        pub account: ethers::core::types::Address,
        pub public_key: ethers::core::types::Bytes,
        pub multiaddr: ethers::core::types::Bytes,
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
        name = "ChannelBumped",
        abi = "ChannelBumped(address,address,bytes32,uint256,uint256)"
    )]
    pub struct ChannelBumpedFilter {
        #[ethevent(indexed)]
        pub source: ethers::core::types::Address,
        #[ethevent(indexed)]
        pub destination: ethers::core::types::Address,
        pub new_commitment: [u8; 32],
        pub ticket_epoch: ethers::core::types::U256,
        pub channel_balance: ethers::core::types::U256,
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
        name = "ChannelFunded",
        abi = "ChannelFunded(address,address,address,uint256)"
    )]
    pub struct ChannelFundedFilter {
        #[ethevent(indexed)]
        pub funder: ethers::core::types::Address,
        #[ethevent(indexed)]
        pub source: ethers::core::types::Address,
        #[ethevent(indexed)]
        pub destination: ethers::core::types::Address,
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
    #[ethevent(name = "ChannelOpened", abi = "ChannelOpened(address,address)")]
    pub struct ChannelOpenedFilter {
        #[ethevent(indexed)]
        pub source: ethers::core::types::Address,
        #[ethevent(indexed)]
        pub destination: ethers::core::types::Address,
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
        name = "ChannelUpdated",
        abi = "ChannelUpdated(address,address,(uint256,bytes32,uint256,uint256,uint8,uint256,uint32))"
    )]
    pub struct ChannelUpdatedFilter {
        #[ethevent(indexed)]
        pub source: ethers::core::types::Address,
        #[ethevent(indexed)]
        pub destination: ethers::core::types::Address,
        pub new_state: Channel,
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
        name = "TicketRedeemed",
        abi = "TicketRedeemed(address,address,bytes32,uint256,uint256,bytes32,uint256,uint256,bytes)"
    )]
    pub struct TicketRedeemedFilter {
        #[ethevent(indexed)]
        pub source: ethers::core::types::Address,
        #[ethevent(indexed)]
        pub destination: ethers::core::types::Address,
        pub next_commitment: [u8; 32],
        pub ticket_epoch: ethers::core::types::U256,
        pub ticket_index: ethers::core::types::U256,
        pub proof_of_relay_secret: [u8; 32],
        pub amount: ethers::core::types::U256,
        pub win_prob: ethers::core::types::U256,
        pub signature: ethers::core::types::Bytes,
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
    #[ethevent(name = "log", abi = "log(string)")]
    pub struct LogFilter(pub String);
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthEvent,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethevent(name = "log_address", abi = "log_address(address)")]
    pub struct LogAddressFilter(pub ethers::core::types::Address);
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthEvent,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethevent(name = "log_array", abi = "log_array(uint256[])")]
    pub struct LogArray1Filter {
        pub val: Vec<ethers::core::types::U256>,
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
    #[ethevent(name = "log_array", abi = "log_array(int256[])")]
    pub struct LogArray2Filter {
        pub val: Vec<I256>,
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
    #[ethevent(name = "log_array", abi = "log_array(address[])")]
    pub struct LogArray3Filter {
        pub val: Vec<ethers::core::types::Address>,
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
    #[ethevent(name = "log_bytes", abi = "log_bytes(bytes)")]
    pub struct LogBytesFilter(pub ethers::core::types::Bytes);
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthEvent,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethevent(name = "log_bytes32", abi = "log_bytes32(bytes32)")]
    pub struct LogBytes32Filter(pub [u8; 32]);
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthEvent,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethevent(name = "log_int", abi = "log_int(int256)")]
    pub struct LogIntFilter(pub I256);
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthEvent,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethevent(name = "log_named_address", abi = "log_named_address(string,address)")]
    pub struct LogNamedAddressFilter {
        pub key: String,
        pub val: ethers::core::types::Address,
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
    #[ethevent(name = "log_named_array", abi = "log_named_array(string,uint256[])")]
    pub struct LogNamedArray1Filter {
        pub key: String,
        pub val: Vec<ethers::core::types::U256>,
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
    #[ethevent(name = "log_named_array", abi = "log_named_array(string,int256[])")]
    pub struct LogNamedArray2Filter {
        pub key: String,
        pub val: Vec<I256>,
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
    #[ethevent(name = "log_named_array", abi = "log_named_array(string,address[])")]
    pub struct LogNamedArray3Filter {
        pub key: String,
        pub val: Vec<ethers::core::types::Address>,
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
    #[ethevent(name = "log_named_bytes", abi = "log_named_bytes(string,bytes)")]
    pub struct LogNamedBytesFilter {
        pub key: String,
        pub val: ethers::core::types::Bytes,
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
    #[ethevent(name = "log_named_bytes32", abi = "log_named_bytes32(string,bytes32)")]
    pub struct LogNamedBytes32Filter {
        pub key: String,
        pub val: [u8; 32],
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
        name = "log_named_decimal_int",
        abi = "log_named_decimal_int(string,int256,uint256)"
    )]
    pub struct LogNamedDecimalIntFilter {
        pub key: String,
        pub val: I256,
        pub decimals: ethers::core::types::U256,
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
        name = "log_named_decimal_uint",
        abi = "log_named_decimal_uint(string,uint256,uint256)"
    )]
    pub struct LogNamedDecimalUintFilter {
        pub key: String,
        pub val: ethers::core::types::U256,
        pub decimals: ethers::core::types::U256,
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
    #[ethevent(name = "log_named_int", abi = "log_named_int(string,int256)")]
    pub struct LogNamedIntFilter {
        pub key: String,
        pub val: I256,
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
    #[ethevent(name = "log_named_string", abi = "log_named_string(string,string)")]
    pub struct LogNamedStringFilter {
        pub key: String,
        pub val: String,
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
    #[ethevent(name = "log_named_uint", abi = "log_named_uint(string,uint256)")]
    pub struct LogNamedUintFilter {
        pub key: String,
        pub val: ethers::core::types::U256,
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
    #[ethevent(name = "log_string", abi = "log_string(string)")]
    pub struct LogStringFilter(pub String);
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthEvent,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethevent(name = "log_uint", abi = "log_uint(uint256)")]
    pub struct LogUintFilter(pub ethers::core::types::U256);
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthEvent,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethevent(name = "logs", abi = "logs(bytes)")]
    pub struct LogsFilter(pub ethers::core::types::Bytes);
    #[derive(Debug, Clone, PartialEq, Eq, ethers :: contract :: EthAbiType)]
    pub enum TicketsUtilsEvents {
        AnnouncementFilter(AnnouncementFilter),
        ChannelBumpedFilter(ChannelBumpedFilter),
        ChannelFundedFilter(ChannelFundedFilter),
        ChannelOpenedFilter(ChannelOpenedFilter),
        ChannelUpdatedFilter(ChannelUpdatedFilter),
        TicketRedeemedFilter(TicketRedeemedFilter),
        LogFilter(LogFilter),
        LogAddressFilter(LogAddressFilter),
        LogArray1Filter(LogArray1Filter),
        LogArray2Filter(LogArray2Filter),
        LogArray3Filter(LogArray3Filter),
        LogBytesFilter(LogBytesFilter),
        LogBytes32Filter(LogBytes32Filter),
        LogIntFilter(LogIntFilter),
        LogNamedAddressFilter(LogNamedAddressFilter),
        LogNamedArray1Filter(LogNamedArray1Filter),
        LogNamedArray2Filter(LogNamedArray2Filter),
        LogNamedArray3Filter(LogNamedArray3Filter),
        LogNamedBytesFilter(LogNamedBytesFilter),
        LogNamedBytes32Filter(LogNamedBytes32Filter),
        LogNamedDecimalIntFilter(LogNamedDecimalIntFilter),
        LogNamedDecimalUintFilter(LogNamedDecimalUintFilter),
        LogNamedIntFilter(LogNamedIntFilter),
        LogNamedStringFilter(LogNamedStringFilter),
        LogNamedUintFilter(LogNamedUintFilter),
        LogStringFilter(LogStringFilter),
        LogUintFilter(LogUintFilter),
        LogsFilter(LogsFilter),
    }
    impl ethers::contract::EthLogDecode for TicketsUtilsEvents {
        fn decode_log(
            log: &ethers::core::abi::RawLog,
        ) -> ::std::result::Result<Self, ethers::core::abi::Error>
        where
            Self: Sized,
        {
            if let Ok(decoded) = AnnouncementFilter::decode_log(log) {
                return Ok(TicketsUtilsEvents::AnnouncementFilter(decoded));
            }
            if let Ok(decoded) = ChannelBumpedFilter::decode_log(log) {
                return Ok(TicketsUtilsEvents::ChannelBumpedFilter(decoded));
            }
            if let Ok(decoded) = ChannelFundedFilter::decode_log(log) {
                return Ok(TicketsUtilsEvents::ChannelFundedFilter(decoded));
            }
            if let Ok(decoded) = ChannelOpenedFilter::decode_log(log) {
                return Ok(TicketsUtilsEvents::ChannelOpenedFilter(decoded));
            }
            if let Ok(decoded) = ChannelUpdatedFilter::decode_log(log) {
                return Ok(TicketsUtilsEvents::ChannelUpdatedFilter(decoded));
            }
            if let Ok(decoded) = TicketRedeemedFilter::decode_log(log) {
                return Ok(TicketsUtilsEvents::TicketRedeemedFilter(decoded));
            }
            if let Ok(decoded) = LogFilter::decode_log(log) {
                return Ok(TicketsUtilsEvents::LogFilter(decoded));
            }
            if let Ok(decoded) = LogAddressFilter::decode_log(log) {
                return Ok(TicketsUtilsEvents::LogAddressFilter(decoded));
            }
            if let Ok(decoded) = LogArray1Filter::decode_log(log) {
                return Ok(TicketsUtilsEvents::LogArray1Filter(decoded));
            }
            if let Ok(decoded) = LogArray2Filter::decode_log(log) {
                return Ok(TicketsUtilsEvents::LogArray2Filter(decoded));
            }
            if let Ok(decoded) = LogArray3Filter::decode_log(log) {
                return Ok(TicketsUtilsEvents::LogArray3Filter(decoded));
            }
            if let Ok(decoded) = LogBytesFilter::decode_log(log) {
                return Ok(TicketsUtilsEvents::LogBytesFilter(decoded));
            }
            if let Ok(decoded) = LogBytes32Filter::decode_log(log) {
                return Ok(TicketsUtilsEvents::LogBytes32Filter(decoded));
            }
            if let Ok(decoded) = LogIntFilter::decode_log(log) {
                return Ok(TicketsUtilsEvents::LogIntFilter(decoded));
            }
            if let Ok(decoded) = LogNamedAddressFilter::decode_log(log) {
                return Ok(TicketsUtilsEvents::LogNamedAddressFilter(decoded));
            }
            if let Ok(decoded) = LogNamedArray1Filter::decode_log(log) {
                return Ok(TicketsUtilsEvents::LogNamedArray1Filter(decoded));
            }
            if let Ok(decoded) = LogNamedArray2Filter::decode_log(log) {
                return Ok(TicketsUtilsEvents::LogNamedArray2Filter(decoded));
            }
            if let Ok(decoded) = LogNamedArray3Filter::decode_log(log) {
                return Ok(TicketsUtilsEvents::LogNamedArray3Filter(decoded));
            }
            if let Ok(decoded) = LogNamedBytesFilter::decode_log(log) {
                return Ok(TicketsUtilsEvents::LogNamedBytesFilter(decoded));
            }
            if let Ok(decoded) = LogNamedBytes32Filter::decode_log(log) {
                return Ok(TicketsUtilsEvents::LogNamedBytes32Filter(decoded));
            }
            if let Ok(decoded) = LogNamedDecimalIntFilter::decode_log(log) {
                return Ok(TicketsUtilsEvents::LogNamedDecimalIntFilter(decoded));
            }
            if let Ok(decoded) = LogNamedDecimalUintFilter::decode_log(log) {
                return Ok(TicketsUtilsEvents::LogNamedDecimalUintFilter(decoded));
            }
            if let Ok(decoded) = LogNamedIntFilter::decode_log(log) {
                return Ok(TicketsUtilsEvents::LogNamedIntFilter(decoded));
            }
            if let Ok(decoded) = LogNamedStringFilter::decode_log(log) {
                return Ok(TicketsUtilsEvents::LogNamedStringFilter(decoded));
            }
            if let Ok(decoded) = LogNamedUintFilter::decode_log(log) {
                return Ok(TicketsUtilsEvents::LogNamedUintFilter(decoded));
            }
            if let Ok(decoded) = LogStringFilter::decode_log(log) {
                return Ok(TicketsUtilsEvents::LogStringFilter(decoded));
            }
            if let Ok(decoded) = LogUintFilter::decode_log(log) {
                return Ok(TicketsUtilsEvents::LogUintFilter(decoded));
            }
            if let Ok(decoded) = LogsFilter::decode_log(log) {
                return Ok(TicketsUtilsEvents::LogsFilter(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData)
        }
    }
    impl ::std::fmt::Display for TicketsUtilsEvents {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                TicketsUtilsEvents::AnnouncementFilter(element) => element.fmt(f),
                TicketsUtilsEvents::ChannelBumpedFilter(element) => element.fmt(f),
                TicketsUtilsEvents::ChannelFundedFilter(element) => element.fmt(f),
                TicketsUtilsEvents::ChannelOpenedFilter(element) => element.fmt(f),
                TicketsUtilsEvents::ChannelUpdatedFilter(element) => element.fmt(f),
                TicketsUtilsEvents::TicketRedeemedFilter(element) => element.fmt(f),
                TicketsUtilsEvents::LogFilter(element) => element.fmt(f),
                TicketsUtilsEvents::LogAddressFilter(element) => element.fmt(f),
                TicketsUtilsEvents::LogArray1Filter(element) => element.fmt(f),
                TicketsUtilsEvents::LogArray2Filter(element) => element.fmt(f),
                TicketsUtilsEvents::LogArray3Filter(element) => element.fmt(f),
                TicketsUtilsEvents::LogBytesFilter(element) => element.fmt(f),
                TicketsUtilsEvents::LogBytes32Filter(element) => element.fmt(f),
                TicketsUtilsEvents::LogIntFilter(element) => element.fmt(f),
                TicketsUtilsEvents::LogNamedAddressFilter(element) => element.fmt(f),
                TicketsUtilsEvents::LogNamedArray1Filter(element) => element.fmt(f),
                TicketsUtilsEvents::LogNamedArray2Filter(element) => element.fmt(f),
                TicketsUtilsEvents::LogNamedArray3Filter(element) => element.fmt(f),
                TicketsUtilsEvents::LogNamedBytesFilter(element) => element.fmt(f),
                TicketsUtilsEvents::LogNamedBytes32Filter(element) => element.fmt(f),
                TicketsUtilsEvents::LogNamedDecimalIntFilter(element) => element.fmt(f),
                TicketsUtilsEvents::LogNamedDecimalUintFilter(element) => element.fmt(f),
                TicketsUtilsEvents::LogNamedIntFilter(element) => element.fmt(f),
                TicketsUtilsEvents::LogNamedStringFilter(element) => element.fmt(f),
                TicketsUtilsEvents::LogNamedUintFilter(element) => element.fmt(f),
                TicketsUtilsEvents::LogStringFilter(element) => element.fmt(f),
                TicketsUtilsEvents::LogUintFilter(element) => element.fmt(f),
                TicketsUtilsEvents::LogsFilter(element) => element.fmt(f),
            }
        }
    }
    #[doc = "Container type for all input parameters for the `IS_SCRIPT` function with signature `IS_SCRIPT()` and selector `[248, 204, 191, 71]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "IS_SCRIPT", abi = "IS_SCRIPT()")]
    pub struct IsScriptCall;
    #[doc = "Container type for all input parameters for the `IS_TEST` function with signature `IS_TEST()` and selector `[250, 118, 38, 212]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "IS_TEST", abi = "IS_TEST()")]
    pub struct IsTestCall;
    #[doc = "Container type for all input parameters for the `PROOF_OF_RELAY_SECRET_0` function with signature `PROOF_OF_RELAY_SECRET_0()` and selector `[67, 233, 58, 203]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "PROOF_OF_RELAY_SECRET_0", abi = "PROOF_OF_RELAY_SECRET_0()")]
    pub struct ProofOfRelaySecret0Call;
    #[doc = "Container type for all input parameters for the `SECRET_0` function with signature `SECRET_0()` and selector `[207, 255, 203, 140]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "SECRET_0", abi = "SECRET_0()")]
    pub struct Secret0Call;
    #[doc = "Container type for all input parameters for the `SECRET_1` function with signature `SECRET_1()` and selector `[94, 147, 146, 85]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "SECRET_1", abi = "SECRET_1()")]
    pub struct Secret1Call;
    #[doc = "Container type for all input parameters for the `SECRET_2` function with signature `SECRET_2()` and selector `[105, 1, 229, 113]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "SECRET_2", abi = "SECRET_2()")]
    pub struct Secret2Call;
    #[doc = "Container type for all input parameters for the `TICKET_AB_LOSS` function with signature `TICKET_AB_LOSS()` and selector `[147, 157, 9, 217]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "TICKET_AB_LOSS", abi = "TICKET_AB_LOSS()")]
    pub struct TicketAbLossCall;
    #[doc = "Container type for all input parameters for the `TICKET_AB_WIN` function with signature `TICKET_AB_WIN()` and selector `[159, 145, 255, 175]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "TICKET_AB_WIN", abi = "TICKET_AB_WIN()")]
    pub struct TicketAbWinCall;
    #[doc = "Container type for all input parameters for the `TICKET_AB_WIN_RECYCLED` function with signature `TICKET_AB_WIN_RECYCLED()` and selector `[156, 148, 111, 166]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "TICKET_AB_WIN_RECYCLED", abi = "TICKET_AB_WIN_RECYCLED()")]
    pub struct TicketAbWinRecycledCall;
    #[doc = "Container type for all input parameters for the `TICKET_BA_WIN` function with signature `TICKET_BA_WIN()` and selector `[215, 106, 26, 83]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "TICKET_BA_WIN", abi = "TICKET_BA_WIN()")]
    pub struct TicketBaWinCall;
    #[doc = "Container type for all input parameters for the `WIN_PROB_0` function with signature `WIN_PROB_0()` and selector `[86, 54, 156, 186]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "WIN_PROB_0", abi = "WIN_PROB_0()")]
    pub struct WinProb0Call;
    #[doc = "Container type for all input parameters for the `WIN_PROB_100` function with signature `WIN_PROB_100()` and selector `[121, 216, 214, 217]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "WIN_PROB_100", abi = "WIN_PROB_100()")]
    pub struct WinProb100Call;
    #[doc = "Container type for all input parameters for the `assertEqChannels` function with signature `assertEqChannels((uint256,bytes32,uint256,uint256,uint8,uint256,uint32),(uint256,bytes32,uint256,uint256,uint8,uint256,uint32))` and selector `[122, 165, 106, 220]`"]
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
        name = "assertEqChannels",
        abi = "assertEqChannels((uint256,bytes32,uint256,uint256,uint8,uint256,uint32),(uint256,bytes32,uint256,uint256,uint8,uint256,uint32))"
    )]
    pub struct AssertEqChannelsCall {
        pub channel_1: Channel,
        pub channel_2: Channel,
    }
    #[doc = "Container type for all input parameters for the `failed` function with signature `failed()` and selector `[186, 65, 79, 166]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "failed", abi = "failed()")]
    pub struct FailedCall;
    #[doc = "Container type for all input parameters for the `getChannelFromTuple` function with signature `getChannelFromTuple(address,bytes32)` and selector `[176, 19, 101, 90]`"]
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
        name = "getChannelFromTuple",
        abi = "getChannelFromTuple(address,bytes32)"
    )]
    pub struct GetChannelFromTupleCall {
        pub hopr_channels: ethers::core::types::Address,
        pub channel_id: [u8; 32],
    }
    #[doc = "Container type for all input parameters for the `getChannelId` function with signature `getChannelId(address,address)` and selector `[123, 10, 237, 61]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "getChannelId", abi = "getChannelId(address,address)")]
    pub struct GetChannelIdCall {
        pub source: ethers::core::types::Address,
        pub destination: ethers::core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `vm` function with signature `vm()` and selector `[58, 118, 132, 99]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "vm", abi = "vm()")]
    pub struct VmCall;
    #[derive(Debug, Clone, PartialEq, Eq, ethers :: contract :: EthAbiType)]
    pub enum TicketsUtilsCalls {
        IsScript(IsScriptCall),
        IsTest(IsTestCall),
        ProofOfRelaySecret0(ProofOfRelaySecret0Call),
        Secret0(Secret0Call),
        Secret1(Secret1Call),
        Secret2(Secret2Call),
        TicketAbLoss(TicketAbLossCall),
        TicketAbWin(TicketAbWinCall),
        TicketAbWinRecycled(TicketAbWinRecycledCall),
        TicketBaWin(TicketBaWinCall),
        WinProb0(WinProb0Call),
        WinProb100(WinProb100Call),
        AssertEqChannels(AssertEqChannelsCall),
        Failed(FailedCall),
        GetChannelFromTuple(GetChannelFromTupleCall),
        GetChannelId(GetChannelIdCall),
        Vm(VmCall),
    }
    impl ethers::core::abi::AbiDecode for TicketsUtilsCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::std::result::Result<Self, ethers::core::abi::AbiError> {
            if let Ok(decoded) =
                <IsScriptCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(TicketsUtilsCalls::IsScript(decoded));
            }
            if let Ok(decoded) = <IsTestCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(TicketsUtilsCalls::IsTest(decoded));
            }
            if let Ok(decoded) =
                <ProofOfRelaySecret0Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(TicketsUtilsCalls::ProofOfRelaySecret0(decoded));
            }
            if let Ok(decoded) =
                <Secret0Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(TicketsUtilsCalls::Secret0(decoded));
            }
            if let Ok(decoded) =
                <Secret1Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(TicketsUtilsCalls::Secret1(decoded));
            }
            if let Ok(decoded) =
                <Secret2Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(TicketsUtilsCalls::Secret2(decoded));
            }
            if let Ok(decoded) =
                <TicketAbLossCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(TicketsUtilsCalls::TicketAbLoss(decoded));
            }
            if let Ok(decoded) =
                <TicketAbWinCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(TicketsUtilsCalls::TicketAbWin(decoded));
            }
            if let Ok(decoded) =
                <TicketAbWinRecycledCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(TicketsUtilsCalls::TicketAbWinRecycled(decoded));
            }
            if let Ok(decoded) =
                <TicketBaWinCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(TicketsUtilsCalls::TicketBaWin(decoded));
            }
            if let Ok(decoded) =
                <WinProb0Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(TicketsUtilsCalls::WinProb0(decoded));
            }
            if let Ok(decoded) =
                <WinProb100Call as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(TicketsUtilsCalls::WinProb100(decoded));
            }
            if let Ok(decoded) =
                <AssertEqChannelsCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(TicketsUtilsCalls::AssertEqChannels(decoded));
            }
            if let Ok(decoded) = <FailedCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(TicketsUtilsCalls::Failed(decoded));
            }
            if let Ok(decoded) =
                <GetChannelFromTupleCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(TicketsUtilsCalls::GetChannelFromTuple(decoded));
            }
            if let Ok(decoded) =
                <GetChannelIdCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(TicketsUtilsCalls::GetChannelId(decoded));
            }
            if let Ok(decoded) = <VmCall as ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(TicketsUtilsCalls::Vm(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ethers::core::abi::AbiEncode for TicketsUtilsCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                TicketsUtilsCalls::IsScript(element) => element.encode(),
                TicketsUtilsCalls::IsTest(element) => element.encode(),
                TicketsUtilsCalls::ProofOfRelaySecret0(element) => element.encode(),
                TicketsUtilsCalls::Secret0(element) => element.encode(),
                TicketsUtilsCalls::Secret1(element) => element.encode(),
                TicketsUtilsCalls::Secret2(element) => element.encode(),
                TicketsUtilsCalls::TicketAbLoss(element) => element.encode(),
                TicketsUtilsCalls::TicketAbWin(element) => element.encode(),
                TicketsUtilsCalls::TicketAbWinRecycled(element) => element.encode(),
                TicketsUtilsCalls::TicketBaWin(element) => element.encode(),
                TicketsUtilsCalls::WinProb0(element) => element.encode(),
                TicketsUtilsCalls::WinProb100(element) => element.encode(),
                TicketsUtilsCalls::AssertEqChannels(element) => element.encode(),
                TicketsUtilsCalls::Failed(element) => element.encode(),
                TicketsUtilsCalls::GetChannelFromTuple(element) => element.encode(),
                TicketsUtilsCalls::GetChannelId(element) => element.encode(),
                TicketsUtilsCalls::Vm(element) => element.encode(),
            }
        }
    }
    impl ::std::fmt::Display for TicketsUtilsCalls {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                TicketsUtilsCalls::IsScript(element) => element.fmt(f),
                TicketsUtilsCalls::IsTest(element) => element.fmt(f),
                TicketsUtilsCalls::ProofOfRelaySecret0(element) => element.fmt(f),
                TicketsUtilsCalls::Secret0(element) => element.fmt(f),
                TicketsUtilsCalls::Secret1(element) => element.fmt(f),
                TicketsUtilsCalls::Secret2(element) => element.fmt(f),
                TicketsUtilsCalls::TicketAbLoss(element) => element.fmt(f),
                TicketsUtilsCalls::TicketAbWin(element) => element.fmt(f),
                TicketsUtilsCalls::TicketAbWinRecycled(element) => element.fmt(f),
                TicketsUtilsCalls::TicketBaWin(element) => element.fmt(f),
                TicketsUtilsCalls::WinProb0(element) => element.fmt(f),
                TicketsUtilsCalls::WinProb100(element) => element.fmt(f),
                TicketsUtilsCalls::AssertEqChannels(element) => element.fmt(f),
                TicketsUtilsCalls::Failed(element) => element.fmt(f),
                TicketsUtilsCalls::GetChannelFromTuple(element) => element.fmt(f),
                TicketsUtilsCalls::GetChannelId(element) => element.fmt(f),
                TicketsUtilsCalls::Vm(element) => element.fmt(f),
            }
        }
    }
    impl ::std::convert::From<IsScriptCall> for TicketsUtilsCalls {
        fn from(var: IsScriptCall) -> Self {
            TicketsUtilsCalls::IsScript(var)
        }
    }
    impl ::std::convert::From<IsTestCall> for TicketsUtilsCalls {
        fn from(var: IsTestCall) -> Self {
            TicketsUtilsCalls::IsTest(var)
        }
    }
    impl ::std::convert::From<ProofOfRelaySecret0Call> for TicketsUtilsCalls {
        fn from(var: ProofOfRelaySecret0Call) -> Self {
            TicketsUtilsCalls::ProofOfRelaySecret0(var)
        }
    }
    impl ::std::convert::From<Secret0Call> for TicketsUtilsCalls {
        fn from(var: Secret0Call) -> Self {
            TicketsUtilsCalls::Secret0(var)
        }
    }
    impl ::std::convert::From<Secret1Call> for TicketsUtilsCalls {
        fn from(var: Secret1Call) -> Self {
            TicketsUtilsCalls::Secret1(var)
        }
    }
    impl ::std::convert::From<Secret2Call> for TicketsUtilsCalls {
        fn from(var: Secret2Call) -> Self {
            TicketsUtilsCalls::Secret2(var)
        }
    }
    impl ::std::convert::From<TicketAbLossCall> for TicketsUtilsCalls {
        fn from(var: TicketAbLossCall) -> Self {
            TicketsUtilsCalls::TicketAbLoss(var)
        }
    }
    impl ::std::convert::From<TicketAbWinCall> for TicketsUtilsCalls {
        fn from(var: TicketAbWinCall) -> Self {
            TicketsUtilsCalls::TicketAbWin(var)
        }
    }
    impl ::std::convert::From<TicketAbWinRecycledCall> for TicketsUtilsCalls {
        fn from(var: TicketAbWinRecycledCall) -> Self {
            TicketsUtilsCalls::TicketAbWinRecycled(var)
        }
    }
    impl ::std::convert::From<TicketBaWinCall> for TicketsUtilsCalls {
        fn from(var: TicketBaWinCall) -> Self {
            TicketsUtilsCalls::TicketBaWin(var)
        }
    }
    impl ::std::convert::From<WinProb0Call> for TicketsUtilsCalls {
        fn from(var: WinProb0Call) -> Self {
            TicketsUtilsCalls::WinProb0(var)
        }
    }
    impl ::std::convert::From<WinProb100Call> for TicketsUtilsCalls {
        fn from(var: WinProb100Call) -> Self {
            TicketsUtilsCalls::WinProb100(var)
        }
    }
    impl ::std::convert::From<AssertEqChannelsCall> for TicketsUtilsCalls {
        fn from(var: AssertEqChannelsCall) -> Self {
            TicketsUtilsCalls::AssertEqChannels(var)
        }
    }
    impl ::std::convert::From<FailedCall> for TicketsUtilsCalls {
        fn from(var: FailedCall) -> Self {
            TicketsUtilsCalls::Failed(var)
        }
    }
    impl ::std::convert::From<GetChannelFromTupleCall> for TicketsUtilsCalls {
        fn from(var: GetChannelFromTupleCall) -> Self {
            TicketsUtilsCalls::GetChannelFromTuple(var)
        }
    }
    impl ::std::convert::From<GetChannelIdCall> for TicketsUtilsCalls {
        fn from(var: GetChannelIdCall) -> Self {
            TicketsUtilsCalls::GetChannelId(var)
        }
    }
    impl ::std::convert::From<VmCall> for TicketsUtilsCalls {
        fn from(var: VmCall) -> Self {
            TicketsUtilsCalls::Vm(var)
        }
    }
    #[doc = "Container type for all return fields from the `IS_SCRIPT` function with signature `IS_SCRIPT()` and selector `[248, 204, 191, 71]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct IsScriptReturn(pub bool);
    #[doc = "Container type for all return fields from the `IS_TEST` function with signature `IS_TEST()` and selector `[250, 118, 38, 212]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct IsTestReturn(pub bool);
    #[doc = "Container type for all return fields from the `PROOF_OF_RELAY_SECRET_0` function with signature `PROOF_OF_RELAY_SECRET_0()` and selector `[67, 233, 58, 203]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct ProofOfRelaySecret0Return(pub [u8; 32]);
    #[doc = "Container type for all return fields from the `SECRET_0` function with signature `SECRET_0()` and selector `[207, 255, 203, 140]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct Secret0Return(pub [u8; 32]);
    #[doc = "Container type for all return fields from the `SECRET_1` function with signature `SECRET_1()` and selector `[94, 147, 146, 85]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct Secret1Return(pub [u8; 32]);
    #[doc = "Container type for all return fields from the `SECRET_2` function with signature `SECRET_2()` and selector `[105, 1, 229, 113]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct Secret2Return(pub [u8; 32]);
    #[doc = "Container type for all return fields from the `TICKET_AB_LOSS` function with signature `TICKET_AB_LOSS()` and selector `[147, 157, 9, 217]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct TicketAbLossReturn {
        pub source: ethers::core::types::Address,
        pub next_commitment: [u8; 32],
        pub ticket_epoch: ethers::core::types::U256,
        pub ticket_index: ethers::core::types::U256,
        pub proof_of_relay_secret: [u8; 32],
        pub amount: ethers::core::types::U256,
        pub win_prob: ethers::core::types::U256,
        pub signature: ethers::core::types::Bytes,
    }
    #[doc = "Container type for all return fields from the `TICKET_AB_WIN` function with signature `TICKET_AB_WIN()` and selector `[159, 145, 255, 175]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct TicketAbWinReturn {
        pub source: ethers::core::types::Address,
        pub next_commitment: [u8; 32],
        pub ticket_epoch: ethers::core::types::U256,
        pub ticket_index: ethers::core::types::U256,
        pub proof_of_relay_secret: [u8; 32],
        pub amount: ethers::core::types::U256,
        pub win_prob: ethers::core::types::U256,
        pub signature: ethers::core::types::Bytes,
    }
    #[doc = "Container type for all return fields from the `TICKET_AB_WIN_RECYCLED` function with signature `TICKET_AB_WIN_RECYCLED()` and selector `[156, 148, 111, 166]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct TicketAbWinRecycledReturn {
        pub source: ethers::core::types::Address,
        pub next_commitment: [u8; 32],
        pub ticket_epoch: ethers::core::types::U256,
        pub ticket_index: ethers::core::types::U256,
        pub proof_of_relay_secret: [u8; 32],
        pub amount: ethers::core::types::U256,
        pub win_prob: ethers::core::types::U256,
        pub signature: ethers::core::types::Bytes,
    }
    #[doc = "Container type for all return fields from the `TICKET_BA_WIN` function with signature `TICKET_BA_WIN()` and selector `[215, 106, 26, 83]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct TicketBaWinReturn {
        pub source: ethers::core::types::Address,
        pub next_commitment: [u8; 32],
        pub ticket_epoch: ethers::core::types::U256,
        pub ticket_index: ethers::core::types::U256,
        pub proof_of_relay_secret: [u8; 32],
        pub amount: ethers::core::types::U256,
        pub win_prob: ethers::core::types::U256,
        pub signature: ethers::core::types::Bytes,
    }
    #[doc = "Container type for all return fields from the `WIN_PROB_0` function with signature `WIN_PROB_0()` and selector `[86, 54, 156, 186]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct WinProb0Return(pub ethers::core::types::U256);
    #[doc = "Container type for all return fields from the `WIN_PROB_100` function with signature `WIN_PROB_100()` and selector `[121, 216, 214, 217]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct WinProb100Return(pub ethers::core::types::U256);
    #[doc = "Container type for all return fields from the `assertEqChannels` function with signature `assertEqChannels((uint256,bytes32,uint256,uint256,uint8,uint256,uint32),(uint256,bytes32,uint256,uint256,uint8,uint256,uint32))` and selector `[122, 165, 106, 220]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct AssertEqChannelsReturn(pub bool);
    #[doc = "Container type for all return fields from the `failed` function with signature `failed()` and selector `[186, 65, 79, 166]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct FailedReturn(pub bool);
    #[doc = "Container type for all return fields from the `getChannelFromTuple` function with signature `getChannelFromTuple(address,bytes32)` and selector `[176, 19, 101, 90]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct GetChannelFromTupleReturn(pub Channel);
    #[doc = "Container type for all return fields from the `getChannelId` function with signature `getChannelId(address,address)` and selector `[123, 10, 237, 61]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct GetChannelIdReturn(pub [u8; 32]);
    #[doc = "Container type for all return fields from the `vm` function with signature `vm()` and selector `[58, 118, 132, 99]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct VmReturn(pub ethers::core::types::Address);
}
