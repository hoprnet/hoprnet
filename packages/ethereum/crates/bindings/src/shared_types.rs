#[doc = "`Log(bytes32[],bytes)`"]
#[derive(
    Clone,
    Debug,
    Default,
    Eq,
    PartialEq,
    ethers :: contract :: EthAbiType,
    ethers :: contract :: EthAbiCodec,
)]
pub struct Log {
    pub topics: Vec<[u8; 32]>,
    pub data: ethers::core::types::Bytes,
}
#[doc = "`Channel(uint256,bytes32,uint256,uint256,uint8,uint256,uint32)`"]
#[derive(
    Clone,
    Debug,
    Default,
    Eq,
    PartialEq,
    ethers :: contract :: EthAbiType,
    ethers :: contract :: EthAbiCodec,
)]
pub struct Channel {
    pub balance: ethers::core::types::U256,
    pub commitment: [u8; 32],
    pub ticket_epoch: ethers::core::types::U256,
    pub ticket_index: ethers::core::types::U256,
    pub status: u8,
    pub channel_epoch: ethers::core::types::U256,
    pub closure_time: u32,
}
