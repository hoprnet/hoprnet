#[cynic::schema("blokli")]
mod schema {}

// https://generator.cynic-rs.dev/

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "QueryRoot")]
pub struct ChannelsQuery {
    pub channels: Vec<Channel>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Channel {
    pub balance: f64,
    pub closure_time: i32,
    pub concrete_channel_id: String,
    pub destination: String,
    pub epoch: i32,
    pub source: String,
    pub status: ChannelStatus,
    pub ticket_index: i32,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Account {
    pub chain_key: String,
    pub multi_addresses: Vec<String>,
    pub packet_key: String,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
pub enum ChannelStatus {
    Open,
    Pendingtoclose,
    Closed,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "QueryRoot")]
pub struct MyQuery {
    // TODO: make this dynamic
    #[arguments(address: "0xabcd")]
    pub hopr_balance: Option<HoprBalance>,
    // TODO: make this dynamic
    #[arguments(address: "0xabcd")]
    pub native_balance: Option<NativeBalance>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct NativeBalance {
    pub address: String,
    pub balance: f64,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct HoprBalance {
    pub address: String,
    pub balance: f64,
}