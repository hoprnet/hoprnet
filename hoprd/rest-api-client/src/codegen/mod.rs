#[allow(unused_imports)]
pub use progenitor_client::{ByteStream, ClientInfo, Error, ResponseValue};
#[allow(unused_imports)]
use progenitor_client::{encode_path, ClientHooks, OperationInfo, RequestBuilderExt};
/// Types used as operation parameters and responses.
#[allow(clippy::all)]
pub mod types {
    /// Error types.
    pub mod error {
        /// Error from a `TryFrom` or `FromStr` implementation.
        pub struct ConversionError(::std::borrow::Cow<'static, str>);
        impl ::std::error::Error for ConversionError {}
        impl ::std::fmt::Display for ConversionError {
            fn fmt(
                &self,
                f: &mut ::std::fmt::Formatter<'_>,
            ) -> Result<(), ::std::fmt::Error> {
                ::std::fmt::Display::fmt(&self.0, f)
            }
        }
        impl ::std::fmt::Debug for ConversionError {
            fn fmt(
                &self,
                f: &mut ::std::fmt::Formatter<'_>,
            ) -> Result<(), ::std::fmt::Error> {
                ::std::fmt::Debug::fmt(&self.0, f)
            }
        }
        impl From<&'static str> for ConversionError {
            fn from(value: &'static str) -> Self {
                Self(value.into())
            }
        }
        impl From<String> for ConversionError {
            fn from(value: String) -> Self {
                Self(value.into())
            }
        }
    }
    ///Contains the node's native addresses.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Contains the node's native addresses.",
    ///  "examples": [
    ///    {
    ///      "native": "0x07eaf07d6624f741e04f4092a755a9027aaab7f6"
    ///    }
    ///  ],
    ///  "type": "object",
    ///  "required": [
    ///    "native"
    ///  ],
    ///  "properties": {
    ///    "native": {
    ///      "examples": [
    ///        "0x07eaf07d6624f741e04f4092a755a9027aaab7f6"
    ///      ],
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct AccountAddressesResponse {
        pub native: ::std::string::String,
    }
    ///Contains all node's and safe's related balances.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Contains all node's and safe's related balances.",
    ///  "examples": [
    ///    {
    ///      "hopr": "1000 wxHOPR",
    ///      "native": "0.1 xDai",
    ///      "safeHopr": "1000 wxHOPR",
    ///      "safeHoprAllowance": "10000 wxHOPR",
    ///      "safeNative": "0.1 xDai"
    ///    }
    ///  ],
    ///  "type": "object",
    ///  "required": [
    ///    "hopr",
    ///    "native",
    ///    "safeHopr",
    ///    "safeHoprAllowance",
    ///    "safeNative"
    ///  ],
    ///  "properties": {
    ///    "hopr": {
    ///      "examples": [
    ///        "2000 wxHOPR"
    ///      ],
    ///      "type": "string"
    ///    },
    ///    "native": {
    ///      "examples": [
    ///        "0.1 xDai"
    ///      ],
    ///      "type": "string"
    ///    },
    ///    "safeHopr": {
    ///      "examples": [
    ///        "2000 wxHOPR"
    ///      ],
    ///      "type": "string"
    ///    },
    ///    "safeHoprAllowance": {
    ///      "examples": [
    ///        "10000 wxHOPR"
    ///      ],
    ///      "type": "string"
    ///    },
    ///    "safeNative": {
    ///      "examples": [
    ///        "0.1 xDai"
    ///      ],
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct AccountBalancesResponse {
        pub hopr: ::std::string::String,
        pub native: ::std::string::String,
        #[serde(rename = "safeHopr")]
        pub safe_hopr: ::std::string::String,
        #[serde(rename = "safeHoprAllowance")]
        pub safe_hopr_allowance: ::std::string::String,
        #[serde(rename = "safeNative")]
        pub safe_native: ::std::string::String,
    }
    ///Represents a peer that has been announced on-chain.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Represents a peer that has been announced on-chain.",
    ///  "examples": [
    ///    {
    ///      "address": "0xb4ce7e6e36ac8b01a974725d5ba730af2b156fbe",
    ///      "multiaddrs": "[/ip4/178.12.1.9/tcp/19092]"
    ///    }
    ///  ],
    ///  "type": "object",
    ///  "required": [
    ///    "address",
    ///    "multiaddrs"
    ///  ],
    ///  "properties": {
    ///    "address": {
    ///      "examples": [
    ///        "0xb4ce7e6e36ac8b01a974725d5ba730af2b156fbe"
    ///      ],
    ///      "type": "string"
    ///    },
    ///    "multiaddrs": {
    ///      "examples": [
    ///        "[/ip4/178.12.1.9/tcp/19092]"
    ///      ],
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct AnnouncedPeer {
        pub address: ::std::string::String,
        pub multiaddrs: ::std::vec::Vec<::std::string::String>,
    }
    ///Standardized error response for the API
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Standardized error response for the API",
    ///  "examples": [
    ///    {
    ///      "status": "INVALID_INPUT",
    ///      "error": "Invalid value passed in parameter 'XYZ'"
    ///    }
    ///  ],
    ///  "type": "object",
    ///  "required": [
    ///    "status"
    ///  ],
    ///  "properties": {
    ///    "error": {
    ///      "examples": [
    ///        "Invalid value passed in parameter 'XYZ'"
    ///      ],
    ///      "type": [
    ///        "string",
    ///        "null"
    ///      ]
    ///    },
    ///    "status": {
    ///      "examples": [
    ///        "INVALID_INPUT"
    ///      ],
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct ApiError {
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub error: ::std::option::Option<::std::string::String>,
        pub status: ::std::string::String,
    }
    ///General information about a channel state.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "General information about a channel state.",
    ///  "examples": [
    ///    {
    ///      "balance": "10 wxHOPR",
    ///      "channelEpoch": 1,
    ///      "channelId": "0x04efc1481d3f106b88527b3844ba40042b823218a9cd29d1aa11c2c2ef8f538f",
    ///      "closureTime": 0,
    ///      "destination": "0x188c4462b75e46f0c7262d7f48d182447b93a93c",
    ///      "source": "0x07eaf07d6624f741e04f4092a755a9027aaab7f6",
    ///      "status": "Open",
    ///      "ticketIndex": 0
    ///    }
    ///  ],
    ///  "type": "object",
    ///  "required": [
    ///    "balance",
    ///    "channelEpoch",
    ///    "channelId",
    ///    "closureTime",
    ///    "destination",
    ///    "source",
    ///    "status",
    ///    "ticketIndex"
    ///  ],
    ///  "properties": {
    ///    "balance": {
    ///      "examples": [
    ///        "10 wxHOPR"
    ///      ],
    ///      "type": "string"
    ///    },
    ///    "channelEpoch": {
    ///      "examples": [
    ///        1
    ///      ],
    ///      "type": "integer",
    ///      "format": "int32",
    ///      "minimum": 0.0
    ///    },
    ///    "channelId": {
    ///      "examples": [
    ///        "0x04efc1481d3f106b88527b3844ba40042b823218a9cd29d1aa11c2c2ef8f538f"
    ///      ],
    ///      "type": "string"
    ///    },
    ///    "closureTime": {
    ///      "examples": [
    ///        0
    ///      ],
    ///      "type": "integer",
    ///      "format": "int64",
    ///      "minimum": 0.0
    ///    },
    ///    "destination": {
    ///      "examples": [
    ///        "0x188c4462b75e46f0c7262d7f48d182447b93a93c"
    ///      ],
    ///      "type": "string"
    ///    },
    ///    "source": {
    ///      "examples": [
    ///        "0x07eaf07d6624f741e04f4092a755a9027aaab7f6"
    ///      ],
    ///      "type": "string"
    ///    },
    ///    "status": {
    ///      "examples": [
    ///        "Open"
    ///      ],
    ///      "type": "string"
    ///    },
    ///    "ticketIndex": {
    ///      "examples": [
    ///        0
    ///      ],
    ///      "type": "integer",
    ///      "format": "int64",
    ///      "minimum": 0.0
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct ChannelInfoResponse {
        pub balance: ::std::string::String,
        #[serde(rename = "channelEpoch")]
        pub channel_epoch: i32,
        #[serde(rename = "channelId")]
        pub channel_id: ::std::string::String,
        #[serde(rename = "closureTime")]
        pub closure_time: i64,
        pub destination: ::std::string::String,
        pub source: ::std::string::String,
        pub status: ::std::string::String,
        #[serde(rename = "ticketIndex")]
        pub ticket_index: i64,
    }
    ///Represents a ticket in a channel.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Represents a ticket in a channel.",
    ///  "examples": [
    ///    {
    ///      "amount": "100",
    ///      "channelEpoch": 1,
    ///      "channelId": "0x04efc1481d3f106b88527b3844ba40042b823218a9cd29d1aa11c2c2ef8f538f",
    ///      "index": 0,
    ///      "indexOffset": 1,
    ///      "signature": "0xe445fcf4e90d25fe3c9199ccfaff85e23ecce8773304d85e7120f1f38787f2329822470487a37f1b5408c8c0b73e874ee9f7594a632713b6096e616857999891",
    ///      "winProb": "1"
    ///    }
    ///  ],
    ///  "type": "object",
    ///  "required": [
    ///    "amount",
    ///    "channelEpoch",
    ///    "channelId",
    ///    "index",
    ///    "signature",
    ///    "winProb"
    ///  ],
    ///  "properties": {
    ///    "amount": {
    ///      "examples": [
    ///        "1.0 wxHOPR"
    ///      ],
    ///      "type": "string"
    ///    },
    ///    "channelEpoch": {
    ///      "examples": [
    ///        1
    ///      ],
    ///      "type": "integer",
    ///      "format": "int32",
    ///      "minimum": 0.0
    ///    },
    ///    "channelId": {
    ///      "examples": [
    ///        "0x04efc1481d3f106b88527b3844ba40042b823218a9cd29d1aa11c2c2ef8f538f"
    ///      ],
    ///      "type": "string"
    ///    },
    ///    "index": {
    ///      "examples": [
    ///        0
    ///      ],
    ///      "type": "integer",
    ///      "format": "int64",
    ///      "minimum": 0.0
    ///    },
    ///    "signature": {
    ///      "examples": [
    ///        "0xe445fcf4e90d25fe3c9199ccfaff85e23ecce8773304d85e7120f1f38787f2329822470487a37f1b5408c8c0b73e874ee9f7594a632713b6096e616857999891"
    ///      ],
    ///      "type": "string"
    ///    },
    ///    "winProb": {
    ///      "examples": [
    ///        "1"
    ///      ],
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct ChannelTicket {
        pub amount: ::std::string::String,
        #[serde(rename = "channelEpoch")]
        pub channel_epoch: i32,
        #[serde(rename = "channelId")]
        pub channel_id: ::std::string::String,
        pub index: i64,
        pub signature: ::std::string::String,
        #[serde(rename = "winProb")]
        pub win_prob: ::std::string::String,
    }
    ///Parameters for enumerating channels.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Parameters for enumerating channels.",
    ///  "examples": [
    ///    {
    ///      "includingClosed": true,
    ///      "fullTopology": false
    ///    }
    ///  ],
    ///  "type": "object",
    ///  "properties": {
    ///    "fullTopology": {
    ///      "description": "Should all channels (not only the ones concerning this node) be enumerated?",
    ///      "default": false,
    ///      "type": "boolean"
    ///    },
    ///    "includingClosed": {
    ///      "description": "Should be the closed channels included?",
    ///      "default": false,
    ///      "type": "boolean"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct ChannelsQueryRequest {
        ///Should all channels (not only the ones concerning this node) be enumerated?
        #[serde(rename = "fullTopology", default)]
        pub full_topology: bool,
        ///Should be the closed channels included?
        #[serde(rename = "includingClosed", default)]
        pub including_closed: bool,
    }
    impl ::std::default::Default for ChannelsQueryRequest {
        fn default() -> Self {
            Self {
                full_topology: Default::default(),
                including_closed: Default::default(),
            }
        }
    }
    ///Status of the channel after a close operation.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Status of the channel after a close operation.",
    ///  "examples": [
    ///    {
    ///      "receipt": "0xd77da7c1821249e663dead1464d185c03223d9663a06bc1d46ed0ad449a07118",
    ///      "channelStatus": "PendingToClose"
    ///    }
    ///  ],
    ///  "type": "object",
    ///  "required": [
    ///    "receipt"
    ///  ],
    ///  "properties": {
    ///    "receipt": {
    ///      "description": "Receipt for the channel close transaction.",
    ///      "examples": [
    ///        "0xd77da7c1821249e663dead1464d185c03223d9663a06bc1d46ed0ad449a07118"
    ///      ],
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct CloseChannelResponse {
        ///Receipt for the channel close transaction.
        pub receipt: ::std::string::String,
    }
    ///Reachable entry node information
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Reachable entry node information",
    ///  "examples": [
    ///    {
    ///      "isEligible": true,
    ///      "multiaddrs": [
    ///        "/ip4/10.0.2.100/tcp/19091"
    ///      ]
    ///    }
    ///  ],
    ///  "type": "object",
    ///  "required": [
    ///    "isEligible",
    ///    "multiaddrs"
    ///  ],
    ///  "properties": {
    ///    "isEligible": {
    ///      "examples": [
    ///        true
    ///      ],
    ///      "type": "boolean"
    ///    },
    ///    "multiaddrs": {
    ///      "examples": [
    ///        [
    ///          "/ip4/10.0.2.100/tcp/19091"
    ///        ]
    ///      ],
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct EntryNode {
        #[serde(rename = "isEligible")]
        pub is_eligible: bool,
        pub multiaddrs: ::std::vec::Vec<::std::string::String>,
    }
    ///Specifies the amount of HOPR tokens to fund a channel with.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Specifies the amount of HOPR tokens to fund a channel with.",
    ///  "examples": [
    ///    {
    ///      "amount": "10 wxHOPR"
    ///    }
    ///  ],
    ///  "type": "object",
    ///  "required": [
    ///    "amount"
    ///  ],
    ///  "properties": {
    ///    "amount": {
    ///      "description": "Amount of HOPR tokens to fund the channel with.",
    ///      "examples": [
    ///        "10 wxHOPR"
    ///      ],
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct FundBodyRequest {
        ///Amount of HOPR tokens to fund the channel with.
        pub amount: ::std::string::String,
    }
    ///Response body for funding a channel.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Response body for funding a channel.",
    ///  "examples": [
    ///    {
    ///      "hash": "0x188c4462b75e46f0c7262d7f48d182447b93a93c"
    ///    }
    ///  ],
    ///  "type": "object",
    ///  "required": [
    ///    "hash"
    ///  ],
    ///  "properties": {
    ///    "hash": {
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct FundChannelResponse {
        pub hash: ::std::string::String,
    }
    ///Heartbeat information for a peer.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Heartbeat information for a peer.",
    ///  "examples": [
    ///    {
    ///      "sent": 10,
    ///      "success": 10
    ///    }
    ///  ],
    ///  "type": "object",
    ///  "required": [
    ///    "sent",
    ///    "success"
    ///  ],
    ///  "properties": {
    ///    "sent": {
    ///      "examples": [
    ///        10
    ///      ],
    ///      "type": "integer",
    ///      "format": "int64",
    ///      "minimum": 0.0
    ///    },
    ///    "success": {
    ///      "examples": [
    ///        10
    ///      ],
    ///      "type": "integer",
    ///      "format": "int64",
    ///      "minimum": 0.0
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct HeartbeatInfo {
        pub sent: i64,
        pub success: i64,
    }
    ///IP transport protocol
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "IP transport protocol",
    ///  "examples": [
    ///    "tcp"
    ///  ],
    ///  "type": "string",
    ///  "enum": [
    ///    "tcp",
    ///    "udp"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum IpProtocol {
        #[serde(rename = "tcp")]
        Tcp,
        #[serde(rename = "udp")]
        Udp,
    }
    impl ::std::fmt::Display for IpProtocol {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Tcp => f.write_str("tcp"),
                Self::Udp => f.write_str("udp"),
            }
        }
    }
    impl ::std::str::FromStr for IpProtocol {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "tcp" => Ok(Self::Tcp),
                "udp" => Ok(Self::Udp),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for IpProtocol {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for IpProtocol {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for IpProtocol {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///Channel information as seen by the node.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Channel information as seen by the node.",
    ///  "examples": [
    ///    {
    ///      "id": "0x04efc1481d3f106b88527b3844ba40042b823218a9cd29d1aa11c2c2ef8f538f",
    ///      "address": "0x188c4462b75e46f0c7262d7f48d182447b93a93c",
    ///      "status": "Open",
    ///      "balance": "10 wxHOPR"
    ///    }
    ///  ],
    ///  "type": "object",
    ///  "required": [
    ///    "balance",
    ///    "id",
    ///    "peerAddress",
    ///    "status"
    ///  ],
    ///  "properties": {
    ///    "balance": {
    ///      "examples": [
    ///        "10 wxHOPR"
    ///      ],
    ///      "type": "string"
    ///    },
    ///    "id": {
    ///      "examples": [
    ///        "0x04efc1481d3f106b88527b3844ba40042b823218a9cd29d1aa11c2c2ef8f538f"
    ///      ],
    ///      "type": "string"
    ///    },
    ///    "peerAddress": {
    ///      "examples": [
    ///        "0x188c4462b75e46f0c7262d7f48d182447b93a93c"
    ///      ],
    ///      "type": "string"
    ///    },
    ///    "status": {
    ///      "examples": [
    ///        "Open"
    ///      ],
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct NodeChannel {
        pub balance: ::std::string::String,
        pub id: ::std::string::String,
        #[serde(rename = "peerAddress")]
        pub peer_address: ::std::string::String,
        pub status: ::std::string::String,
    }
    ///Listing of channels.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Listing of channels.",
    ///  "examples": [
    ///    {
    ///      "all": [
    ///        {
    ///          "channelId": "0x04efc1481d3f106b88527b3844ba40042b823218a9cd29d1aa11c2c2ef8f538f",
    ///          "source": "0x07eaf07d6624f741e04f4092a755a9027aaab7f6",
    ///          "destination": "0x188c4462b75e46f0c7262d7f48d182447b93a93c",
    ///          "balance": "10 wxHOPR",
    ///          "status": "Open",
    ///          "ticketIndex": 0,
    ///          "channelEpoch": 1,
    ///          "closureTime": 0
    ///        }
    ///      ],
    ///      "incoming": [],
    ///      "outgoing": [
    ///        {
    ///          "balance": "10 wxHOPR",
    ///          "id": "0x04efc1481d3f106b88527b3844ba40042b823218a9cd29d1aa11c2c2ef8f538f",
    ///          "peerAddress": "0x188c4462b75e46f0c7262d7f48d182447b93a93c",
    ///          "status": "Open"
    ///        }
    ///      ]
    ///    }
    ///  ],
    ///  "type": "object",
    ///  "required": [
    ///    "all",
    ///    "incoming",
    ///    "outgoing"
    ///  ],
    ///  "properties": {
    ///    "all": {
    ///      "description": "Complete channel topology as seen by this node.",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/ChannelInfoResponse"
    ///      }
    ///    },
    ///    "incoming": {
    ///      "description": "Channels incoming to this node.",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/NodeChannel"
    ///      }
    ///    },
    ///    "outgoing": {
    ///      "description": "Channels outgoing from this node.",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/NodeChannel"
    ///      }
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct NodeChannelsResponse {
        ///Complete channel topology as seen by this node.
        pub all: ::std::vec::Vec<ChannelInfoResponse>,
        ///Channels incoming to this node.
        pub incoming: ::std::vec::Vec<NodeChannel>,
        ///Channels outgoing from this node.
        pub outgoing: ::std::vec::Vec<NodeChannel>,
    }
    /**Information about the current node. Covers network, addresses, eligibility, connectivity status, contracts addresses
and indexer state.*/
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Information about the current node. Covers network, addresses, eligibility, connectivity status, contracts addresses\nand indexer state.",
    ///  "examples": [
    ///    {
    ///      "announcedAddress": [
    ///        "/ip4/10.0.2.100/tcp/19092"
    ///      ],
    ///      "providerUrl": "https://staging.blokli.hoprnet.link",
    ///      "hoprNetworkName": "rotsee",
    ///      "channelClosurePeriod": 15,
    ///      "connectivityStatus": "Green",
    ///      "hoprNodeSafe": "0x42bc901b1d040f984ed626eff550718498a6798a",
    ///      "listeningAddress": [
    ///        "/ip4/10.0.2.100/tcp/19092"
    ///      ]
    ///    }
    ///  ],
    ///  "type": "object",
    ///  "required": [
    ///    "announcedAddress",
    ///    "channelClosurePeriod",
    ///    "connectivityStatus",
    ///    "hoprNetworkName",
    ///    "hoprNodeSafe",
    ///    "listeningAddress",
    ///    "providerUrl"
    ///  ],
    ///  "properties": {
    ///    "announcedAddress": {
    ///      "examples": [
    ///        [
    ///          "/ip4/10.0.2.100/tcp/19092"
    ///        ]
    ///      ],
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "channelClosurePeriod": {
    ///      "description": "Channel closure period in seconds",
    ///      "examples": [
    ///        15
    ///      ],
    ///      "type": "integer",
    ///      "format": "int64",
    ///      "minimum": 0.0
    ///    },
    ///    "connectivityStatus": {
    ///      "examples": [
    ///        "Green"
    ///      ],
    ///      "type": "string"
    ///    },
    ///    "hoprNetworkName": {
    ///      "examples": [
    ///        "rotsee"
    ///      ],
    ///      "type": "string"
    ///    },
    ///    "hoprNodeSafe": {
    ///      "examples": [
    ///        "0x42bc901b1d040f984ed626eff550718498a6798a"
    ///      ],
    ///      "type": "string"
    ///    },
    ///    "listeningAddress": {
    ///      "examples": [
    ///        [
    ///          "/ip4/10.0.2.100/tcp/19092"
    ///        ]
    ///      ],
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "providerUrl": {
    ///      "examples": [
    ///        "https://staging.blokli.hoprnet.link"
    ///      ],
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct NodeInfoResponse {
        #[serde(rename = "announcedAddress")]
        pub announced_address: ::std::vec::Vec<::std::string::String>,
        ///Channel closure period in seconds
        #[serde(rename = "channelClosurePeriod")]
        pub channel_closure_period: i64,
        #[serde(rename = "connectivityStatus")]
        pub connectivity_status: ::std::string::String,
        #[serde(rename = "hoprNetworkName")]
        pub hopr_network_name: ::std::string::String,
        #[serde(rename = "hoprNodeSafe")]
        pub hopr_node_safe: ::std::string::String,
        #[serde(rename = "listeningAddress")]
        pub listening_address: ::std::vec::Vec<::std::string::String>,
        #[serde(rename = "providerUrl")]
        pub provider_url: ::std::string::String,
    }
    ///Contains the multiaddresses of peers that are `announced` on-chain and `observed` by the node.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Contains the multiaddresses of peers that are `announced` on-chain and `observed` by the node.",
    ///  "examples": [
    ///    {
    ///      "announced": [
    ///        "/ip4/10.0.2.100/tcp/19093"
    ///      ],
    ///      "observed": [
    ///        "/ip4/10.0.2.100/tcp/19093"
    ///      ]
    ///    }
    ///  ],
    ///  "type": "object",
    ///  "required": [
    ///    "announced",
    ///    "observed"
    ///  ],
    ///  "properties": {
    ///    "announced": {
    ///      "examples": [
    ///        [
    ///          "/ip4/10.0.2.100/tcp/19093"
    ///        ]
    ///      ],
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "observed": {
    ///      "examples": [
    ///        [
    ///          "/ip4/10.0.2.100/tcp/19093"
    ///        ]
    ///      ],
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct NodePeerInfoResponse {
        pub announced: ::std::vec::Vec<::std::string::String>,
        pub observed: ::std::vec::Vec<::std::string::String>,
    }
    ///Quality information for a peer.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Quality information for a peer.",
    ///  "examples": [
    ///    {
    ///      "quality": 0.7
    ///    }
    ///  ],
    ///  "type": "object",
    ///  "properties": {
    ///    "score": {
    ///      "description": "Minimum peer quality to be included in the response.",
    ///      "examples": [
    ///        0.7
    ///      ],
    ///      "type": "number",
    ///      "format": "double"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct NodePeersQueryRequest {
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub score: ::std::option::Option<f64>,
    }
    impl ::std::default::Default for NodePeersQueryRequest {
        fn default() -> Self {
            Self { score: Default::default() }
        }
    }
    ///All connected and announced peers.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "All connected and announced peers.",
    ///  "examples": [
    ///    {
    ///      "connected": [
    ///        {
    ///          "address": "0xb4ce7e6e36ac8b01a974725d5ba730af2b156fbe",
    ///          "multiaddr": "/ip4/178.12.1.9/tcp/19092",
    ///          "heartbeats": {
    ///            "sent": 10,
    ///            "success": 10
    ///          },
    ///          "lastSeen": 1690000000,
    ///          "lastSeenLatency": 100,
    ///          "quality": 0.7,
    ///          "backoff": 0.5,
    ///          "isNew": true
    ///        }
    ///      ],
    ///      "announced": [
    ///        {
    ///          "address": "0xb4ce7e6e36ac8b01a974725d5ba730af2b156fbe",
    ///          "multiaddr": "/ip4/178.12.1.9/tcp/19092"
    ///        }
    ///      ]
    ///    }
    ///  ],
    ///  "type": "object",
    ///  "required": [
    ///    "announced",
    ///    "connected"
    ///  ],
    ///  "properties": {
    ///    "announced": {
    ///      "examples": [
    ///        [
    ///          {
    ///            "address": "0xb4ce7e6e36ac8b01a974725d5ba730af2b156fbe",
    ///            "multiaddr": "/ip4/178.12.1.9/tcp/19092"
    ///          }
    ///        ]
    ///      ],
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/AnnouncedPeer"
    ///      }
    ///    },
    ///    "connected": {
    ///      "examples": [
    ///        [
    ///          {
    ///            "address": "0xb4ce7e6e36ac8b01a974725d5ba730af2b156fbe",
    ///            "multiaddr": "/ip4/178.12.1.9/tcp/19092",
    ///            "heartbeats": {
    ///              "sent": 10,
    ///              "success": 10
    ///            },
    ///            "lastSeen": 1690000000,
    ///            "lastSeenLatency": 100,
    ///            "quality": 0.7,
    ///            "backoff": 0.5,
    ///            "isNew": true
    ///          }
    ///        ]
    ///      ],
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/PeerObservations"
    ///      }
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct NodePeersResponse {
        pub announced: ::std::vec::Vec<AnnouncedPeer>,
        pub connected: ::std::vec::Vec<PeerObservations>,
    }
    ///Received tickets statistics.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Received tickets statistics.",
    ///  "examples": [
    ///    {
    ///      "winningCount": 0,
    ///      "neglectedValue": "0 wxHOPR",
    ///      "redeemedValue": "1000 wxHOPR",
    ///      "rejectedValue": "0 wxHOPR",
    ///      "unredeemedValue": "2000 wxHOPR"
    ///    }
    ///  ],
    ///  "type": "object",
    ///  "required": [
    ///    "neglectedValue",
    ///    "redeemedValue",
    ///    "rejectedValue",
    ///    "unredeemedValue",
    ///    "winningCount"
    ///  ],
    ///  "properties": {
    ///    "neglectedValue": {
    ///      "examples": [
    ///        "0 wxHOPR"
    ///      ],
    ///      "type": "string"
    ///    },
    ///    "redeemedValue": {
    ///      "examples": [
    ///        "100 wxHOPR"
    ///      ],
    ///      "type": "string"
    ///    },
    ///    "rejectedValue": {
    ///      "examples": [
    ///        "0 wHOPR"
    ///      ],
    ///      "type": "string"
    ///    },
    ///    "unredeemedValue": {
    ///      "examples": [
    ///        "20 wxHOPR"
    ///      ],
    ///      "type": "string"
    ///    },
    ///    "winningCount": {
    ///      "examples": [
    ///        0
    ///      ],
    ///      "type": "integer",
    ///      "format": "int64",
    ///      "minimum": 0.0
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct NodeTicketStatisticsResponse {
        #[serde(rename = "neglectedValue")]
        pub neglected_value: ::std::string::String,
        #[serde(rename = "redeemedValue")]
        pub redeemed_value: ::std::string::String,
        #[serde(rename = "rejectedValue")]
        pub rejected_value: ::std::string::String,
        #[serde(rename = "unredeemedValue")]
        pub unredeemed_value: ::std::string::String,
        #[serde(rename = "winningCount")]
        pub winning_count: i64,
    }
    ///Running node version.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Running node version.",
    ///  "examples": [
    ///    {
    ///      "version": "2.1.0"
    ///    }
    ///  ],
    ///  "type": "object",
    ///  "required": [
    ///    "version"
    ///  ],
    ///  "properties": {
    ///    "version": {
    ///      "examples": [
    ///        "2.1.0"
    ///      ],
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct NodeVersionResponse {
        pub version: ::std::string::String,
    }
    ///Request body for opening a channel.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Request body for opening a channel.",
    ///  "examples": [
    ///    {
    ///      "amount": "10 wxHOPR",
    ///      "destination": "0xa8194d36e322592d4c707b70dbe96121f5c74c64"
    ///    }
    ///  ],
    ///  "type": "object",
    ///  "required": [
    ///    "amount",
    ///    "destination"
    ///  ],
    ///  "properties": {
    ///    "amount": {
    ///      "description": "Initial amount of stake in HOPR tokens.",
    ///      "examples": [
    ///        "10 wxHOPR"
    ///      ],
    ///      "type": "string"
    ///    },
    ///    "destination": {
    ///      "description": "On-chain address of the counterparty.",
    ///      "examples": [
    ///        "0xa8194d36e322592d4c707b70dbe96121f5c74c64"
    ///      ],
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct OpenChannelBodyRequest {
        ///Initial amount of stake in HOPR tokens.
        pub amount: ::std::string::String,
        ///On-chain address of the counterparty.
        pub destination: ::std::string::String,
    }
    ///Response body for opening a channel.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Response body for opening a channel.",
    ///  "examples": [
    ///    {
    ///      "channelId": "0x04efc1481d3f106b88527b3844ba40042b823218a9cd29d1aa11c2c2ef8f538f",
    ///      "transactionReceipt": "0x5181ac24759b8e01b3c932e4636c3852f386d17517a8dfc640a5ba6f2258f29c"
    ///    }
    ///  ],
    ///  "type": "object",
    ///  "required": [
    ///    "channelId",
    ///    "transactionReceipt"
    ///  ],
    ///  "properties": {
    ///    "channelId": {
    ///      "description": "ID of the new channel.",
    ///      "examples": [
    ///        "0x04efc1481d3f106b88527b3844ba40042b823218a9cd29d1aa11c2c2ef8f538f"
    ///      ],
    ///      "type": "string"
    ///    },
    ///    "transactionReceipt": {
    ///      "description": "Receipt of the channel open transaction.",
    ///      "examples": [
    ///        "0x5181ac24759b8e01b3c932e4636c3852f386d17517a8dfc640a5ba6f2258f29c"
    ///      ],
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct OpenChannelResponse {
        ///ID of the new channel.
        #[serde(rename = "channelId")]
        pub channel_id: ::std::string::String,
        ///Receipt of the channel open transaction.
        #[serde(rename = "transactionReceipt")]
        pub transaction_receipt: ::std::string::String,
    }
    ///All information about a known peer.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "All information about a known peer.",
    ///  "examples": [
    ///    {
    ///      "address": "0xb4ce7e6e36ac8b01a974725d5ba730af2b156fbe",
    ///      "multiaddr": "/ip4/178.12.1.9/tcp/19092",
    ///      "probeRate": 0.476,
    ///      "lastSeen": 1690000000,
    ///      "averageLatency": 100,
    ///      "score": 0.7,
    ///      "packetStats": {
    ///        "packetsOut": 100,
    ///        "packetsIn": 50,
    ///        "bytesOut": 102400,
    ///        "bytesIn": 51200
    ///      }
    ///    }
    ///  ],
    ///  "type": "object",
    ///  "required": [
    ///    "address",
    ///    "averageLatency",
    ///    "lastUpdate",
    ///    "probeRate",
    ///    "score"
    ///  ],
    ///  "properties": {
    ///    "address": {
    ///      "examples": [
    ///        "0xb4ce7e6e36ac8b01a974725d5ba730af2b156fbe"
    ///      ],
    ///      "type": "string"
    ///    },
    ///    "averageLatency": {
    ///      "examples": [
    ///        100
    ///      ],
    ///      "type": "integer",
    ///      "minimum": 0.0
    ///    },
    ///    "lastUpdate": {
    ///      "examples": [
    ///        1690000000
    ///      ],
    ///      "type": "integer",
    ///      "minimum": 0.0
    ///    },
    ///    "multiaddr": {
    ///      "examples": [
    ///        "/ip4/178.12.1.9/tcp/19092"
    ///      ],
    ///      "type": [
    ///        "string",
    ///        "null"
    ///      ]
    ///    },
    ///    "probeRate": {
    ///      "examples": [
    ///        0.476
    ///      ],
    ///      "type": "number",
    ///      "format": "double"
    ///    },
    ///    "score": {
    ///      "examples": [
    ///        0.7
    ///      ],
    ///      "type": "number",
    ///      "format": "double"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct PeerObservations {
        pub address: ::std::string::String,
        #[serde(rename = "averageLatency")]
        pub average_latency: u64,
        #[serde(rename = "lastUpdate")]
        pub last_update: u64,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub multiaddr: ::std::option::Option<::std::string::String>,
        #[serde(rename = "probeRate")]
        pub probe_rate: f64,
        pub score: f64,
    }
    ///Packet statistics for a peer.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Packet statistics for a peer.",
    ///  "examples": [
    ///    {
    ///      "packetsOut": 100,
    ///      "packetsIn": 50,
    ///      "bytesOut": 102400,
    ///      "bytesIn": 51200
    ///    }
    ///  ],
    ///  "type": "object",
    ///  "required": [
    ///    "bytesIn",
    ///    "bytesOut",
    ///    "packetsIn",
    ///    "packetsOut"
    ///  ],
    ///  "properties": {
    ///    "bytesIn": {
    ///      "examples": [
    ///        51200
    ///      ],
    ///      "type": "integer",
    ///      "format": "int64",
    ///      "minimum": 0.0
    ///    },
    ///    "bytesOut": {
    ///      "examples": [
    ///        102400
    ///      ],
    ///      "type": "integer",
    ///      "format": "int64",
    ///      "minimum": 0.0
    ///    },
    ///    "packetsIn": {
    ///      "examples": [
    ///        50
    ///      ],
    ///      "type": "integer",
    ///      "format": "int64",
    ///      "minimum": 0.0
    ///    },
    ///    "packetsOut": {
    ///      "examples": [
    ///        100
    ///      ],
    ///      "type": "integer",
    ///      "format": "int64",
    ///      "minimum": 0.0
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct PeerPacketStatsResponse {
        #[serde(rename = "bytesIn")]
        pub bytes_in: i64,
        #[serde(rename = "bytesOut")]
        pub bytes_out: i64,
        #[serde(rename = "packetsIn")]
        pub packets_in: i64,
        #[serde(rename = "packetsOut")]
        pub packets_out: i64,
    }
    ///Contains the latency and the reported version of a peer that has been pinged.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Contains the latency and the reported version of a peer that has been pinged.",
    ///  "examples": [
    ///    {
    ///      "latency": 200
    ///    }
    ///  ],
    ///  "type": "object",
    ///  "required": [
    ///    "latency"
    ///  ],
    ///  "properties": {
    ///    "latency": {
    ///      "examples": [
    ///        200
    ///      ],
    ///      "type": "integer",
    ///      "format": "int64",
    ///      "minimum": 0.0
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct PingResponse {
        pub latency: i64,
    }
    ///Request parameters for creating a websocket session.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Request parameters for creating a websocket session.",
    ///  "examples": [
    ///    {
    ///      "Hops": 1
    ///    }
    ///  ],
    ///  "oneOf": [
    ///    {
    ///      "type": "object",
    ///      "required": [
    ///        "Hops"
    ///      ],
    ///      "properties": {
    ///        "Hops": {
    ///          "type": "integer",
    ///          "minimum": 0.0
    ///        }
    ///      }
    ///    }
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub enum RoutingOptions {
        Hops(u64),
    }
    impl ::std::convert::From<u64> for RoutingOptions {
        fn from(value: u64) -> Self {
            Self::Hops(value)
        }
    }
    ///Session capabilities that can be negotiated with the target peer.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Session capabilities that can be negotiated with the target peer.",
    ///  "examples": [
    ///    "Segmentation"
    ///  ],
    ///  "type": "string",
    ///  "enum": [
    ///    "Segmentation",
    ///    "Retransmission",
    ///    "RetransmissionAckOnly",
    ///    "NoDelay",
    ///    "NoRateControl"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum SessionCapability {
        Segmentation,
        Retransmission,
        RetransmissionAckOnly,
        NoDelay,
        NoRateControl,
    }
    impl ::std::fmt::Display for SessionCapability {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Segmentation => f.write_str("Segmentation"),
                Self::Retransmission => f.write_str("Retransmission"),
                Self::RetransmissionAckOnly => f.write_str("RetransmissionAckOnly"),
                Self::NoDelay => f.write_str("NoDelay"),
                Self::NoRateControl => f.write_str("NoRateControl"),
            }
        }
    }
    impl ::std::str::FromStr for SessionCapability {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "Segmentation" => Ok(Self::Segmentation),
                "Retransmission" => Ok(Self::Retransmission),
                "RetransmissionAckOnly" => Ok(Self::RetransmissionAckOnly),
                "NoDelay" => Ok(Self::NoDelay),
                "NoRateControl" => Ok(Self::NoRateControl),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for SessionCapability {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for SessionCapability {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for SessionCapability {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///Request body for creating a new client session.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Request body for creating a new client session.",
    ///  "examples": [
    ///    {
    ///      "destination": "0x1B482420Afa04aeC1Ef0e4a00C18451E84466c75",
    ///      "forwardPath": {
    ///        "Hops": 1
    ///      },
    ///      "returnPath": {
    ///        "Hops": 1
    ///      },
    ///      "target": {
    ///        "Plain": "localhost:8080"
    ///      },
    ///      "listenHost": "127.0.0.1:10000",
    ///      "capabilities": [
    ///        "Retransmission",
    ///        "Segmentation"
    ///      ],
    ///      "responseBuffer": "2 MB",
    ///      "maxSurbUpstream": "2000 kb/s",
    ///      "sessionPool": 0,
    ///      "maxClientSessions": 2
    ///    }
    ///  ],
    ///  "type": "object",
    ///  "required": [
    ///    "destination",
    ///    "forwardPath",
    ///    "returnPath",
    ///    "target"
    ///  ],
    ///  "properties": {
    ///    "capabilities": {
    ///      "description": "Capabilities for the Session protocol.\n\nDefaults to `Segmentation` and `Retransmission` for TCP and nothing for UDP.",
    ///      "type": [
    ///        "array",
    ///        "null"
    ///      ],
    ///      "items": {
    ///        "$ref": "#/components/schemas/SessionCapability"
    ///      }
    ///    },
    ///    "destination": {
    ///      "description": "Address of the Exit node.",
    ///      "type": "string"
    ///    },
    ///    "forwardPath": {
    ///      "$ref": "#/components/schemas/RoutingOptions"
    ///    },
    ///    "listenHost": {
    ///      "description": "Listen host (`ip:port`) for the Session socket at the Entry node.\n\nSupports also partial specification (only `ip` or only `:port`) with the\nrespective part replaced by the node's configured default.",
    ///      "type": [
    ///        "string",
    ///        "null"
    ///      ]
    ///    },
    ///    "maxClientSessions": {
    ///      "description": "The maximum number of client sessions that the listener can spawn.\n\nThis currently applies only to the TCP sessions, as UDP sessions cannot\nhandle multiple clients (and spawn therefore always only a single session).\n\nIf this value is smaller than the value specified in `session_pool`, it will\nbe set to that value.\n\nThe default value is 5.",
    ///      "type": [
    ///        "integer",
    ///        "null"
    ///      ],
    ///      "minimum": 0.0
    ///    },
    ///    "maxSurbUpstream": {
    ///      "description": "The maximum throughput at which artificial SURBs might be generated and sent\nto the recipient of the Session.\n\nOn Sessions that rarely send data but receive a lot (= Exit node has high SURB consumption),\nthis should roughly match the maximum retrieval throughput.\n\nAll syntaxes like \"2 MBps\", \"1.2Mbps\", \"300 kb/s\", \"1.23 Mb/s\" are supported.",
    ///      "type": [
    ///        "string",
    ///        "null"
    ///      ]
    ///    },
    ///    "responseBuffer": {
    ///      "description": "The amount of response data the Session counterparty can deliver back to us,\nwithout us sending any SURBs to them.\n\nIn other words, this size is recalculated to a number of SURBs delivered\nto the counterparty upfront and then maintained.\nThe maintenance is dynamic, based on the number of responses we receive.\n\nAll syntaxes like \"2 MB\", \"128 kiB\", \"3MiB\" are supported. The value must be\nat least the size of 2 Session packet payloads.",
    ///      "type": [
    ///        "string",
    ///        "null"
    ///      ]
    ///    },
    ///    "returnPath": {
    ///      "$ref": "#/components/schemas/RoutingOptions"
    ///    },
    ///    "sessionPool": {
    ///      "description": "How many Sessions to pool for clients.\n\nIf no sessions are pooled, they will be opened ad-hoc when a client connects.\nIt has no effect on UDP sessions in the current implementation.\n\nCurrently, the maximum value is 5.",
    ///      "type": [
    ///        "integer",
    ///        "null"
    ///      ],
    ///      "minimum": 0.0
    ///    },
    ///    "target": {
    ///      "$ref": "#/components/schemas/SessionTargetSpec"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct SessionClientRequest {
        /**Capabilities for the Session protocol.

Defaults to `Segmentation` and `Retransmission` for TCP and nothing for UDP.*/
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub capabilities: ::std::option::Option<::std::vec::Vec<SessionCapability>>,
        ///Address of the Exit node.
        pub destination: ::std::string::String,
        #[serde(rename = "forwardPath")]
        pub forward_path: RoutingOptions,
        /**Listen host (`ip:port`) for the Session socket at the Entry node.

Supports also partial specification (only `ip` or only `:port`) with the
respective part replaced by the node's configured default.*/
        #[serde(
            rename = "listenHost",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub listen_host: ::std::option::Option<::std::string::String>,
        /**The maximum number of client sessions that the listener can spawn.

This currently applies only to the TCP sessions, as UDP sessions cannot
handle multiple clients (and spawn therefore always only a single session).

If this value is smaller than the value specified in `session_pool`, it will
be set to that value.

The default value is 5.*/
        #[serde(
            rename = "maxClientSessions",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub max_client_sessions: ::std::option::Option<u64>,
        /**The maximum throughput at which artificial SURBs might be generated and sent
to the recipient of the Session.

On Sessions that rarely send data but receive a lot (= Exit node has high SURB consumption),
this should roughly match the maximum retrieval throughput.

All syntaxes like "2 MBps", "1.2Mbps", "300 kb/s", "1.23 Mb/s" are supported.*/
        #[serde(
            rename = "maxSurbUpstream",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub max_surb_upstream: ::std::option::Option<::std::string::String>,
        /**The amount of response data the Session counterparty can deliver back to us,
without us sending any SURBs to them.

In other words, this size is recalculated to a number of SURBs delivered
to the counterparty upfront and then maintained.
The maintenance is dynamic, based on the number of responses we receive.

All syntaxes like "2 MB", "128 kiB", "3MiB" are supported. The value must be
at least the size of 2 Session packet payloads.*/
        #[serde(
            rename = "responseBuffer",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub response_buffer: ::std::option::Option<::std::string::String>,
        #[serde(rename = "returnPath")]
        pub return_path: RoutingOptions,
        /**How many Sessions to pool for clients.

If no sessions are pooled, they will be opened ad-hoc when a client connects.
It has no effect on UDP sessions in the current implementation.

Currently, the maximum value is 5.*/
        #[serde(
            rename = "sessionPool",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub session_pool: ::std::option::Option<u64>,
        pub target: SessionTargetSpec,
    }
    ///Response body for creating a new client session.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Response body for creating a new client session.",
    ///  "examples": [
    ///    {
    ///      "target": "example.com:80",
    ///      "destination": "0x5112D584a1C72Fc250176B57aEba5fFbbB287D8F",
    ///      "forwardPath": {
    ///        "Hops": 1
    ///      },
    ///      "returnPath": {
    ///        "Hops": 1
    ///      },
    ///      "protocol": "tcp",
    ///      "ip": "127.0.0.1",
    ///      "port": 5542,
    ///      "hoprMtu": 1002,
    ///      "surbLen": 398,
    ///      "activeClients": [],
    ///      "maxClientSessions": 2,
    ///      "maxSurbUpstream": "2000 kb/s",
    ///      "responseBuffer": "2 MB",
    ///      "sessionPool": 0
    ///    }
    ///  ],
    ///  "type": "object",
    ///  "required": [
    ///    "activeClients",
    ///    "destination",
    ///    "forwardPath",
    ///    "hoprMtu",
    ///    "ip",
    ///    "maxClientSessions",
    ///    "port",
    ///    "protocol",
    ///    "returnPath",
    ///    "surbLen",
    ///    "target"
    ///  ],
    ///  "properties": {
    ///    "activeClients": {
    ///      "description": "Lists Session IDs of all active clients.\n\nCan contain multiple entries on TCP sessions, but currently\nalways only a single entry on UDP sessions.",
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "destination": {
    ///      "description": "Destination node (exit node) of the Session.",
    ///      "type": "string"
    ///    },
    ///    "forwardPath": {
    ///      "$ref": "#/components/schemas/RoutingOptions"
    ///    },
    ///    "hoprMtu": {
    ///      "description": "MTU used by the underlying HOPR transport.",
    ///      "type": "integer",
    ///      "minimum": 0.0
    ///    },
    ///    "ip": {
    ///      "description": "Listening IP address of the Session's socket.",
    ///      "examples": [
    ///        "127.0.0.1"
    ///      ],
    ///      "type": "string"
    ///    },
    ///    "maxClientSessions": {
    ///      "description": "The maximum number of client sessions that the listener can spawn.\n\nThis currently applies only to the TCP sessions, as UDP sessions cannot\nhave multiple clients (defaults to 1 for UDP).",
    ///      "type": "integer",
    ///      "minimum": 0.0
    ///    },
    ///    "maxSurbUpstream": {
    ///      "description": "The maximum throughput at which artificial SURBs might be generated and sent\nto the recipient of the Session.",
    ///      "type": [
    ///        "string",
    ///        "null"
    ///      ]
    ///    },
    ///    "port": {
    ///      "description": "Listening port of the Session's socket.",
    ///      "examples": [
    ///        5542
    ///      ],
    ///      "type": "integer",
    ///      "format": "int32",
    ///      "minimum": 0.0
    ///    },
    ///    "protocol": {
    ///      "$ref": "#/components/schemas/IpProtocol"
    ///    },
    ///    "responseBuffer": {
    ///      "description": "The amount of response data the Session counterparty can deliver back to us, without us\nsending any SURBs to them.",
    ///      "type": [
    ///        "string",
    ///        "null"
    ///      ]
    ///    },
    ///    "returnPath": {
    ///      "$ref": "#/components/schemas/RoutingOptions"
    ///    },
    ///    "sessionPool": {
    ///      "description": "How many Sessions to pool for clients.",
    ///      "type": [
    ///        "integer",
    ///        "null"
    ///      ],
    ///      "minimum": 0.0
    ///    },
    ///    "surbLen": {
    ///      "description": "Size of a Single Use Reply Block used by the protocol.\n\nThis is useful for SURB balancing calculations.",
    ///      "type": "integer",
    ///      "minimum": 0.0
    ///    },
    ///    "target": {
    ///      "description": "Target of the Session.",
    ///      "examples": [
    ///        "example.com:80"
    ///      ],
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct SessionClientResponse {
        /**Lists Session IDs of all active clients.

Can contain multiple entries on TCP sessions, but currently
always only a single entry on UDP sessions.*/
        #[serde(rename = "activeClients")]
        pub active_clients: ::std::vec::Vec<::std::string::String>,
        ///Destination node (exit node) of the Session.
        pub destination: ::std::string::String,
        #[serde(rename = "forwardPath")]
        pub forward_path: RoutingOptions,
        ///MTU used by the underlying HOPR transport.
        #[serde(rename = "hoprMtu")]
        pub hopr_mtu: u64,
        ///Listening IP address of the Session's socket.
        pub ip: ::std::string::String,
        /**The maximum number of client sessions that the listener can spawn.

This currently applies only to the TCP sessions, as UDP sessions cannot
have multiple clients (defaults to 1 for UDP).*/
        #[serde(rename = "maxClientSessions")]
        pub max_client_sessions: u64,
        /**The maximum throughput at which artificial SURBs might be generated and sent
to the recipient of the Session.*/
        #[serde(
            rename = "maxSurbUpstream",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub max_surb_upstream: ::std::option::Option<::std::string::String>,
        ///Listening port of the Session's socket.
        pub port: i32,
        pub protocol: IpProtocol,
        /**The amount of response data the Session counterparty can deliver back to us, without us
sending any SURBs to them.*/
        #[serde(
            rename = "responseBuffer",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub response_buffer: ::std::option::Option<::std::string::String>,
        #[serde(rename = "returnPath")]
        pub return_path: RoutingOptions,
        ///How many Sessions to pool for clients.
        #[serde(
            rename = "sessionPool",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub session_pool: ::std::option::Option<u64>,
        /**Size of a Single Use Reply Block used by the protocol.

This is useful for SURB balancing calculations.*/
        #[serde(rename = "surbLen")]
        pub surb_len: u64,
        ///Target of the Session.
        pub target: ::std::string::String,
    }
    ///`SessionConfig`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "examples": [
    ///    {
    ///      "responseBuffer": "2 MB",
    ///      "maxSurbUpstream": "2 Mbps"
    ///    }
    ///  ],
    ///  "type": "object",
    ///  "properties": {
    ///    "maxSurbUpstream": {
    ///      "description": "The maximum throughput at which artificial SURBs might be generated and sent\nto the recipient of the Session.\n\nOn Sessions that rarely send data but receive a lot (= Exit node has high SURB consumption),\nthis should roughly match the maximum retrieval throughput.\n\nAll syntaxes like \"2 MBps\", \"1.2Mbps\", \"300 kb/s\", \"1.23 Mb/s\" are supported.",
    ///      "type": "string"
    ///    },
    ///    "responseBuffer": {
    ///      "description": "The amount of response data the Session counterparty can deliver back to us,\nwithout us sending any SURBs to them.\n\nIn other words, this size is recalculated to a number of SURBs delivered\nto the counterparty upfront and then maintained.\nThe maintenance is dynamic, based on the number of responses we receive.\n\nAll syntaxes like \"2 MB\", \"128 kiB\", \"3MiB\" are supported. The value must be\nat least the size of 2 Session packet payloads.",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct SessionConfig {
        /**The maximum throughput at which artificial SURBs might be generated and sent
to the recipient of the Session.

On Sessions that rarely send data but receive a lot (= Exit node has high SURB consumption),
this should roughly match the maximum retrieval throughput.

All syntaxes like "2 MBps", "1.2Mbps", "300 kb/s", "1.23 Mb/s" are supported.*/
        #[serde(
            rename = "maxSurbUpstream",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub max_surb_upstream: ::std::option::Option<::std::string::String>,
        /**The amount of response data the Session counterparty can deliver back to us,
without us sending any SURBs to them.

In other words, this size is recalculated to a number of SURBs delivered
to the counterparty upfront and then maintained.
The maintenance is dynamic, based on the number of responses we receive.

All syntaxes like "2 MB", "128 kiB", "3MiB" are supported. The value must be
at least the size of 2 Session packet payloads.*/
        #[serde(
            rename = "responseBuffer",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub response_buffer: ::std::option::Option<::std::string::String>,
    }
    impl ::std::default::Default for SessionConfig {
        fn default() -> Self {
            Self {
                max_surb_upstream: Default::default(),
                response_buffer: Default::default(),
            }
        }
    }
    ///Session acknowledgement metrics.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Session acknowledgement metrics.",
    ///  "type": "object",
    ///  "required": [
    ///    "incomingAcknowledgedFrames",
    ///    "incomingRetransmissionRequests",
    ///    "incomingSegments",
    ///    "mode",
    ///    "outgoingAcknowledgedFrames",
    ///    "outgoingRetransmissionRequests",
    ///    "outgoingSegments"
    ///  ],
    ///  "properties": {
    ///    "incomingAcknowledgedFrames": {
    ///      "description": "Total incoming frame acknowledgements.",
    ///      "type": "integer",
    ///      "format": "int64",
    ///      "minimum": 0.0
    ///    },
    ///    "incomingRetransmissionRequests": {
    ///      "description": "Total incoming retransmission requests received.",
    ///      "type": "integer",
    ///      "format": "int64",
    ///      "minimum": 0.0
    ///    },
    ///    "incomingSegments": {
    ///      "description": "Total incoming segments received.",
    ///      "type": "integer",
    ///      "format": "int64",
    ///      "minimum": 0.0
    ///    },
    ///    "mode": {
    ///      "$ref": "#/components/schemas/SessionStatsAckMode"
    ///    },
    ///    "outgoingAcknowledgedFrames": {
    ///      "description": "Total outgoing frames acknowledgements",
    ///      "type": "integer",
    ///      "format": "int64",
    ///      "minimum": 0.0
    ///    },
    ///    "outgoingRetransmissionRequests": {
    ///      "description": "Total outgoing retransmission requests received.",
    ///      "type": "integer",
    ///      "format": "int64",
    ///      "minimum": 0.0
    ///    },
    ///    "outgoingSegments": {
    ///      "description": "Total outgoing segments sent.",
    ///      "type": "integer",
    ///      "format": "int64",
    ///      "minimum": 0.0
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct SessionStatsAck {
        ///Total incoming frame acknowledgements.
        #[serde(rename = "incomingAcknowledgedFrames")]
        pub incoming_acknowledged_frames: i64,
        ///Total incoming retransmission requests received.
        #[serde(rename = "incomingRetransmissionRequests")]
        pub incoming_retransmission_requests: i64,
        ///Total incoming segments received.
        #[serde(rename = "incomingSegments")]
        pub incoming_segments: i64,
        pub mode: SessionStatsAckMode,
        ///Total outgoing frames acknowledgements
        #[serde(rename = "outgoingAcknowledgedFrames")]
        pub outgoing_acknowledged_frames: i64,
        ///Total outgoing retransmission requests received.
        #[serde(rename = "outgoingRetransmissionRequests")]
        pub outgoing_retransmission_requests: i64,
        ///Total outgoing segments sent.
        #[serde(rename = "outgoingSegments")]
        pub outgoing_segments: i64,
    }
    ///Session acknowledgement mode for metrics.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Session acknowledgement mode for metrics.",
    ///  "type": "string",
    ///  "enum": [
    ///    "none",
    ///    "partial",
    ///    "full",
    ///    "both"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum SessionStatsAckMode {
        #[serde(rename = "none")]
        None,
        #[serde(rename = "partial")]
        Partial,
        #[serde(rename = "full")]
        Full,
        #[serde(rename = "both")]
        Both,
    }
    impl ::std::fmt::Display for SessionStatsAckMode {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::None => f.write_str("none"),
                Self::Partial => f.write_str("partial"),
                Self::Full => f.write_str("full"),
                Self::Both => f.write_str("both"),
            }
        }
    }
    impl ::std::str::FromStr for SessionStatsAckMode {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "none" => Ok(Self::None),
                "partial" => Ok(Self::Partial),
                "full" => Ok(Self::Full),
                "both" => Ok(Self::Both),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for SessionStatsAckMode {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for SessionStatsAckMode {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for SessionStatsAckMode {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///Session frame buffer metrics.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Session frame buffer metrics.",
    ///  "type": "object",
    ///  "required": [
    ///    "frameCapacity",
    ///    "frameMtu",
    ///    "frameTimeoutMs",
    ///    "framesCompleted",
    ///    "framesDiscarded",
    ///    "framesEmitted",
    ///    "incompleteFrames"
    ///  ],
    ///  "properties": {
    ///    "frameCapacity": {
    ///      "description": "Configured capacity of the frame buffer.",
    ///      "type": "integer",
    ///      "minimum": 0.0
    ///    },
    ///    "frameMtu": {
    ///      "description": "Maximum Transmission Unit for frames.",
    ///      "type": "integer",
    ///      "minimum": 0.0
    ///    },
    ///    "frameTimeoutMs": {
    ///      "description": "Configured timeout for frame reassembly/acknowledgement (in milliseconds).",
    ///      "type": "integer",
    ///      "format": "int64",
    ///      "minimum": 0.0
    ///    },
    ///    "framesCompleted": {
    ///      "description": "Total number of frames successfully completed/assembled.",
    ///      "type": "integer",
    ///      "format": "int64",
    ///      "minimum": 0.0
    ///    },
    ///    "framesDiscarded": {
    ///      "description": "Total number of frames discarded (e.g. due to timeout or errors).",
    ///      "type": "integer",
    ///      "format": "int64",
    ///      "minimum": 0.0
    ///    },
    ///    "framesEmitted": {
    ///      "description": "Total number of frames emitted to the application.",
    ///      "type": "integer",
    ///      "format": "int64",
    ///      "minimum": 0.0
    ///    },
    ///    "incompleteFrames": {
    ///      "description": "Number of frames currently being assembled (incomplete).",
    ///      "type": "integer",
    ///      "minimum": 0.0
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct SessionStatsFrameBuffer {
        ///Configured capacity of the frame buffer.
        #[serde(rename = "frameCapacity")]
        pub frame_capacity: u64,
        ///Maximum Transmission Unit for frames.
        #[serde(rename = "frameMtu")]
        pub frame_mtu: u64,
        ///Configured timeout for frame reassembly/acknowledgement (in milliseconds).
        #[serde(rename = "frameTimeoutMs")]
        pub frame_timeout_ms: i64,
        ///Total number of frames successfully completed/assembled.
        #[serde(rename = "framesCompleted")]
        pub frames_completed: i64,
        ///Total number of frames discarded (e.g. due to timeout or errors).
        #[serde(rename = "framesDiscarded")]
        pub frames_discarded: i64,
        ///Total number of frames emitted to the application.
        #[serde(rename = "framesEmitted")]
        pub frames_emitted: i64,
        ///Number of frames currently being assembled (incomplete).
        #[serde(rename = "incompleteFrames")]
        pub incomplete_frames: u64,
    }
    ///Session lifetime metrics.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Session lifetime metrics.",
    ///  "type": "object",
    ///  "required": [
    ///    "createdAtMs",
    ///    "idleMs",
    ///    "lastActivityAtMs",
    ///    "state",
    ///    "uptimeMs"
    ///  ],
    ///  "properties": {
    ///    "createdAtMs": {
    ///      "description": "Time when the session was created (in milliseconds since UNIX epoch).",
    ///      "type": "integer",
    ///      "format": "int64",
    ///      "minimum": 0.0
    ///    },
    ///    "idleMs": {
    ///      "description": "Duration since the last activity (in milliseconds).",
    ///      "type": "integer",
    ///      "format": "int64",
    ///      "minimum": 0.0
    ///    },
    ///    "lastActivityAtMs": {
    ///      "description": "Time of the last read or write activity (in milliseconds since UNIX epoch).",
    ///      "type": "integer",
    ///      "format": "int64",
    ///      "minimum": 0.0
    ///    },
    ///    "state": {
    ///      "$ref": "#/components/schemas/SessionStatsState"
    ///    },
    ///    "uptimeMs": {
    ///      "description": "Total duration the session has been alive (in milliseconds).",
    ///      "type": "integer",
    ///      "format": "int64",
    ///      "minimum": 0.0
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct SessionStatsLifetime {
        ///Time when the session was created (in milliseconds since UNIX epoch).
        #[serde(rename = "createdAtMs")]
        pub created_at_ms: i64,
        ///Duration since the last activity (in milliseconds).
        #[serde(rename = "idleMs")]
        pub idle_ms: i64,
        ///Time of the last read or write activity (in milliseconds since UNIX epoch).
        #[serde(rename = "lastActivityAtMs")]
        pub last_activity_at_ms: i64,
        pub state: SessionStatsState,
        ///Total duration the session has been alive (in milliseconds).
        #[serde(rename = "uptimeMs")]
        pub uptime_ms: i64,
    }
    ///Complete snapshot of session metrics.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Complete snapshot of session metrics.",
    ///  "type": "object",
    ///  "required": [
    ///    "ack",
    ///    "frameBuffer",
    ///    "lifetime",
    ///    "sessionId",
    ///    "snapshotAtMs",
    ///    "surb",
    ///    "transport"
    ///  ],
    ///  "properties": {
    ///    "ack": {
    ///      "$ref": "#/components/schemas/SessionStatsAck"
    ///    },
    ///    "frameBuffer": {
    ///      "$ref": "#/components/schemas/SessionStatsFrameBuffer"
    ///    },
    ///    "lifetime": {
    ///      "$ref": "#/components/schemas/SessionStatsLifetime"
    ///    },
    ///    "sessionId": {
    ///      "description": "The session ID.",
    ///      "type": "string"
    ///    },
    ///    "snapshotAtMs": {
    ///      "description": "Time when this snapshot was taken (in milliseconds since UNIX epoch).",
    ///      "type": "integer",
    ///      "format": "int64",
    ///      "minimum": 0.0
    ///    },
    ///    "surb": {
    ///      "$ref": "#/components/schemas/SessionStatsSurb"
    ///    },
    ///    "transport": {
    ///      "$ref": "#/components/schemas/SessionStatsTransport"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct SessionStatsResponse {
        pub ack: SessionStatsAck,
        #[serde(rename = "frameBuffer")]
        pub frame_buffer: SessionStatsFrameBuffer,
        pub lifetime: SessionStatsLifetime,
        ///The session ID.
        #[serde(rename = "sessionId")]
        pub session_id: ::std::string::String,
        ///Time when this snapshot was taken (in milliseconds since UNIX epoch).
        #[serde(rename = "snapshotAtMs")]
        pub snapshot_at_ms: i64,
        pub surb: SessionStatsSurb,
        pub transport: SessionStatsTransport,
    }
    ///Session lifecycle state for metrics.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Session lifecycle state for metrics.",
    ///  "type": "string",
    ///  "enum": [
    ///    "active",
    ///    "closing",
    ///    "closed"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum SessionStatsState {
        #[serde(rename = "active")]
        Active,
        #[serde(rename = "closing")]
        Closing,
        #[serde(rename = "closed")]
        Closed,
    }
    impl ::std::fmt::Display for SessionStatsState {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Active => f.write_str("active"),
                Self::Closing => f.write_str("closing"),
                Self::Closed => f.write_str("closed"),
            }
        }
    }
    impl ::std::str::FromStr for SessionStatsState {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "active" => Ok(Self::Active),
                "closing" => Ok(Self::Closing),
                "closed" => Ok(Self::Closed),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for SessionStatsState {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for SessionStatsState {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for SessionStatsState {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///Session SURB (Single Use Reply Block) metrics.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Session SURB (Single Use Reply Block) metrics.",
    ///  "type": "object",
    ///  "required": [
    ///    "bufferEstimate",
    ///    "consumedTotal",
    ///    "producedTotal",
    ///    "ratePerSec",
    ///    "refillInFlight"
    ///  ],
    ///  "properties": {
    ///    "bufferEstimate": {
    ///      "description": "Estimated number of SURBs currently available.",
    ///      "type": "integer",
    ///      "format": "int64",
    ///      "minimum": 0.0
    ///    },
    ///    "consumedTotal": {
    ///      "description": "Total SURBs consumed/used.",
    ///      "type": "integer",
    ///      "format": "int64",
    ///      "minimum": 0.0
    ///    },
    ///    "producedTotal": {
    ///      "description": "Total SURBs produced/minted.",
    ///      "type": "integer",
    ///      "format": "int64",
    ///      "minimum": 0.0
    ///    },
    ///    "ratePerSec": {
    ///      "description": "Rate of SURB consumption/production per second.",
    ///      "type": "number",
    ///      "format": "double"
    ///    },
    ///    "refillInFlight": {
    ///      "description": "Whether a SURB refill request is currently in flight.",
    ///      "type": "boolean"
    ///    },
    ///    "targetBuffer": {
    ///      "description": "Target number of SURBs to maintain in buffer (if configured).",
    ///      "type": [
    ///        "integer",
    ///        "null"
    ///      ],
    ///      "format": "int64",
    ///      "minimum": 0.0
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct SessionStatsSurb {
        ///Estimated number of SURBs currently available.
        #[serde(rename = "bufferEstimate")]
        pub buffer_estimate: i64,
        ///Total SURBs consumed/used.
        #[serde(rename = "consumedTotal")]
        pub consumed_total: i64,
        ///Total SURBs produced/minted.
        #[serde(rename = "producedTotal")]
        pub produced_total: i64,
        #[serde(rename = "ratePerSec")]
        pub rate_per_sec: f64,
        ///Whether a SURB refill request is currently in flight.
        #[serde(rename = "refillInFlight")]
        pub refill_in_flight: bool,
        ///Target number of SURBs to maintain in buffer (if configured).
        #[serde(
            rename = "targetBuffer",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub target_buffer: ::std::option::Option<i64>,
    }
    ///Session transport-level metrics.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Session transport-level metrics.",
    ///  "type": "object",
    ///  "required": [
    ///    "bytesIn",
    ///    "bytesOut",
    ///    "packetsIn",
    ///    "packetsOut"
    ///  ],
    ///  "properties": {
    ///    "bytesIn": {
    ///      "description": "Total bytes received.",
    ///      "type": "integer",
    ///      "format": "int64",
    ///      "minimum": 0.0
    ///    },
    ///    "bytesOut": {
    ///      "description": "Total bytes sent.",
    ///      "type": "integer",
    ///      "format": "int64",
    ///      "minimum": 0.0
    ///    },
    ///    "packetsIn": {
    ///      "description": "Total packets received.",
    ///      "type": "integer",
    ///      "format": "int64",
    ///      "minimum": 0.0
    ///    },
    ///    "packetsOut": {
    ///      "description": "Total packets sent.",
    ///      "type": "integer",
    ///      "format": "int64",
    ///      "minimum": 0.0
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct SessionStatsTransport {
        ///Total bytes received.
        #[serde(rename = "bytesIn")]
        pub bytes_in: i64,
        ///Total bytes sent.
        #[serde(rename = "bytesOut")]
        pub bytes_out: i64,
        ///Total packets received.
        #[serde(rename = "packetsIn")]
        pub packets_in: i64,
        ///Total packets sent.
        #[serde(rename = "packetsOut")]
        pub packets_out: i64,
    }
    ///Session target specification.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Session target specification.",
    ///  "examples": [
    ///    {
    ///      "Service": 0
    ///    }
    ///  ],
    ///  "oneOf": [
    ///    {
    ///      "type": "object",
    ///      "required": [
    ///        "Plain"
    ///      ],
    ///      "properties": {
    ///        "Plain": {
    ///          "type": "string"
    ///        }
    ///      }
    ///    },
    ///    {
    ///      "type": "object",
    ///      "required": [
    ///        "Sealed"
    ///      ],
    ///      "properties": {
    ///        "Sealed": {
    ///          "type": "array",
    ///          "items": {
    ///            "type": "integer",
    ///            "format": "int32",
    ///            "minimum": 0.0
    ///          }
    ///        }
    ///      }
    ///    },
    ///    {
    ///      "type": "object",
    ///      "required": [
    ///        "Service"
    ///      ],
    ///      "properties": {
    ///        "Service": {
    ///          "type": "integer",
    ///          "format": "int32",
    ///          "minimum": 0.0
    ///        }
    ///      }
    ///    }
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub enum SessionTargetSpec {
        Plain(::std::string::String),
        Sealed(::std::vec::Vec<i32>),
        Service(i32),
    }
    impl ::std::convert::From<::std::vec::Vec<i32>> for SessionTargetSpec {
        fn from(value: ::std::vec::Vec<i32>) -> Self {
            Self::Sealed(value)
        }
    }
    impl ::std::convert::From<i32> for SessionTargetSpec {
        fn from(value: i32) -> Self {
            Self::Service(value)
        }
    }
    ///Contains the ticket price in HOPR tokens.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Contains the ticket price in HOPR tokens.",
    ///  "examples": [
    ///    {
    ///      "price": "0.03 wxHOPR"
    ///    }
    ///  ],
    ///  "type": "object",
    ///  "required": [
    ///    "price"
    ///  ],
    ///  "properties": {
    ///    "price": {
    ///      "description": "Price of the ticket in HOPR tokens.",
    ///      "examples": [
    ///        "0.03 wxHOPR"
    ///      ],
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct TicketPriceResponse {
        ///Price of the ticket in HOPR tokens.
        pub price: ::std::string::String,
    }
    ///Contains the winning probability of a ticket.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Contains the winning probability of a ticket.",
    ///  "examples": [
    ///    {
    ///      "probability": 0.5
    ///    }
    ///  ],
    ///  "type": "object",
    ///  "required": [
    ///    "probability"
    ///  ],
    ///  "properties": {
    ///    "probability": {
    ///      "description": "Winning probability of a ticket.",
    ///      "examples": [
    ///        0.5
    ///      ],
    ///      "type": "number",
    ///      "format": "double"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct TicketProbabilityResponse {
        pub probability: f64,
    }
    ///Request body for the withdrawal endpoint.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Request body for the withdrawal endpoint.",
    ///  "examples": [
    ///    {
    ///      "address": "0xb4ce7e6e36ac8b01a974725d5ba730af2b156fbe",
    ///      "amount": "20000 wxHOPR"
    ///    }
    ///  ],
    ///  "type": "object",
    ///  "required": [
    ///    "address",
    ///    "amount"
    ///  ],
    ///  "properties": {
    ///    "address": {
    ///      "examples": [
    ///        "0xb4ce7e6e36ac8b01a974725d5ba730af2b156fbe"
    ///      ],
    ///      "type": "string"
    ///    },
    ///    "amount": {
    ///      "examples": [
    ///        "20000 wxHOPR"
    ///      ],
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct WithdrawBodyRequest {
        pub address: ::std::string::String,
        pub amount: ::std::string::String,
    }
    ///Response body for the withdrawal endpoint.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Response body for the withdrawal endpoint.",
    ///  "examples": [
    ///    {
    ///      "receipt": "0xb4ce7e6e36ac8b01a974725d5ba730af2b156fbe"
    ///    }
    ///  ],
    ///  "type": "object",
    ///  "required": [
    ///    "receipt"
    ///  ],
    ///  "properties": {
    ///    "receipt": {
    ///      "examples": [
    ///        "0xb4ce7e6e36ac8b01a974725d5ba730af2b156fbe"
    ///      ],
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct WithdrawResponse {
        pub receipt: ::std::string::String,
    }
}
#[derive(Clone, Debug)]
/**Client for hoprd-api

API enabling developers to interact with a hoprd node programatically through HTTP REST API.

Version: 4.6.0*/
pub struct Client {
    pub(crate) baseurl: String,
    pub(crate) client: reqwest::Client,
}
impl Client {
    /// Create a new client.
    ///
    /// `baseurl` is the base URL provided to the internal
    /// `reqwest::Client`, and should include a scheme and hostname,
    /// as well as port and a path stem if applicable.
    pub fn new(baseurl: &str) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let client = {
            let dur = ::std::time::Duration::from_secs(15u64);
            reqwest::ClientBuilder::new().connect_timeout(dur).timeout(dur)
        };
        #[cfg(target_arch = "wasm32")]
        let client = reqwest::ClientBuilder::new();
        Self::new_with_client(baseurl, client.build().unwrap())
    }
    /// Construct a new client with an existing `reqwest::Client`,
    /// allowing more control over its configuration.
    ///
    /// `baseurl` is the base URL provided to the internal
    /// `reqwest::Client`, and should include a scheme and hostname,
    /// as well as port and a path stem if applicable.
    pub fn new_with_client(baseurl: &str, client: reqwest::Client) -> Self {
        Self {
            baseurl: baseurl.to_string(),
            client,
        }
    }
}
impl ClientInfo<()> for Client {
    fn api_version() -> &'static str {
        "4.6.0"
    }
    fn baseurl(&self) -> &str {
        self.baseurl.as_str()
    }
    fn client(&self) -> &reqwest::Client {
        &self.client
    }
    fn inner(&self) -> &() {
        &()
    }
}
impl ClientHooks<()> for &Client {}
#[allow(clippy::all)]
impl Client {
    /**Get node's native addresses

Sends a `GET` request to `/api/v4/account/addresses`

*/
    pub async fn addresses<'a>(
        &'a self,
    ) -> Result<ResponseValue<types::AccountAddressesResponse>, Error<()>> {
        let url = format!("{}/api/v4/account/addresses", self.baseurl,);
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .get(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "addresses",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Get node's and associated Safe's HOPR and native balances as the allowance for HOPR
tokens to be drawn by HoprChannels from Safe

HOPR tokens from the Safe balance are used to fund the payment channels between this
node and other nodes on the network.
NATIVE balance of the Node is used to pay for the gas fees for the blockchain.

Sends a `GET` request to `/api/v4/account/balances`

*/
    pub async fn balances<'a>(
        &'a self,
    ) -> Result<ResponseValue<types::AccountBalancesResponse>, Error<()>> {
        let url = format!("{}/api/v4/account/balances", self.baseurl,);
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .get(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "balances",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Withdraw funds from this node to the ethereum wallet address

Withdraw funds from this node to the ethereum wallet address

Sends a `POST` request to `/api/v4/account/withdraw`

Arguments:
- `body`: Request body for the withdraw endpoint
*/
    pub async fn withdraw<'a>(
        &'a self,
        body: &'a types::WithdrawBodyRequest,
    ) -> Result<ResponseValue<types::WithdrawResponse>, Error<()>> {
        let url = format!("{}/api/v4/account/withdraw", self.baseurl,);
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .post(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .json(&body)
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "withdraw",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Lists channels opened to/from this node. Alternatively, it can print all
the channels in the network as this node sees them

List channels opened to/from this node. Alternatively, it can print all the channels in the network as this node sees them.

Sends a `GET` request to `/api/v4/channels`

Arguments:
- `full_topology`: Should all channels (not only the ones concerning this node) be enumerated?
- `including_closed`: Should be the closed channels included?
*/
    pub async fn list_channels<'a>(
        &'a self,
        full_topology: Option<bool>,
        including_closed: Option<bool>,
    ) -> Result<ResponseValue<types::NodeChannelsResponse>, Error<()>> {
        let url = format!("{}/api/v4/channels", self.baseurl,);
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .get(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .query(&progenitor_client::QueryParam::new("fullTopology", &full_topology))
            .query(
                &progenitor_client::QueryParam::new("includingClosed", &including_closed),
            )
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "list_channels",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Opens a channel to the given on-chain address with the given initial stake of HOPR tokens

Opens a channel to the given on-chain address with the given initial stake of HOPR tokens.

Sends a `POST` request to `/api/v4/channels`

Arguments:
- `body`: Open channel request specification: on-chain address of the counterparty and the initial HOPR token stake.
*/
    pub async fn open_channel<'a>(
        &'a self,
        body: &'a types::OpenChannelBodyRequest,
    ) -> Result<ResponseValue<types::OpenChannelResponse>, Error<()>> {
        let url = format!("{}/api/v4/channels", self.baseurl,);
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .post(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .json(&body)
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "open_channel",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            201u16 => ResponseValue::from_response(response).await,
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Returns information about the given channel

Returns information about the given channel.

Sends a `GET` request to `/api/v4/channels/{channelId}`

Arguments:
- `channel_id`: ID of the channel.
*/
    pub async fn show_channel<'a>(
        &'a self,
        channel_id: &'a str,
    ) -> Result<ResponseValue<types::ChannelInfoResponse>, Error<()>> {
        let url = format!(
            "{}/api/v4/channels/{}", self.baseurl, encode_path(& channel_id.to_string()),
        );
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .get(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "show_channel",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Closes the given channel

Closes the given channel.

Sends a `DELETE` request to `/api/v4/channels/{channelId}`

Arguments:
- `channel_id`: ID of the channel.
*/
    pub async fn close_channel<'a>(
        &'a self,
        channel_id: &'a str,
    ) -> Result<ResponseValue<types::CloseChannelResponse>, Error<()>> {
        let url = format!(
            "{}/api/v4/channels/{}", self.baseurl, encode_path(& channel_id.to_string()),
        );
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .delete(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "close_channel",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Funds the given channel with the given amount of HOPR tokens

Funds the given channel with the given amount of HOPR tokens.

Sends a `POST` request to `/api/v4/channels/{channelId}/fund`

Arguments:
- `channel_id`: ID of the channel.
- `body`: Specifies the amount of HOPR tokens to fund a channel with.
*/
    pub async fn fund_channel<'a>(
        &'a self,
        channel_id: &'a str,
        body: &'a types::FundBodyRequest,
    ) -> Result<ResponseValue<types::FundChannelResponse>, Error<()>> {
        let url = format!(
            "{}/api/v4/channels/{}/fund", self.baseurl, encode_path(& channel_id
            .to_string()),
        );
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .post(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .json(&body)
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "fund_channel",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Lists all tickets for the given channel  ID

Lists all tickets for the given channel ID.

Sends a `GET` request to `/api/v4/channels/{channelId}/tickets`

Arguments:
- `channel_id`: ID of the channel.
*/
    pub async fn show_channel_tickets<'a>(
        &'a self,
        channel_id: &'a str,
    ) -> Result<ResponseValue<::std::vec::Vec<types::ChannelTicket>>, Error<()>> {
        let url = format!(
            "{}/api/v4/channels/{}/tickets", self.baseurl, encode_path(& channel_id
            .to_string()),
        );
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .get(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "show_channel_tickets",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Starts redeeming all tickets in the given channel

Starts redeeming all tickets in the given channel.

Sends a `POST` request to `/api/v4/channels/{channelId}/tickets/redeem`

Arguments:
- `channel_id`: ID of the channel.
*/
    pub async fn redeem_tickets_in_channel<'a>(
        &'a self,
        channel_id: &'a str,
    ) -> Result<ResponseValue<ByteStream>, Error<types::ApiError>> {
        let url = format!(
            "{}/api/v4/channels/{}/tickets/redeem", self.baseurl, encode_path(&
            channel_id.to_string()),
        );
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self.client.post(url).headers(header_map).build()?;
        let info = OperationInfo {
            operation_id: "redeem_tickets_in_channel",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200..=299 => Ok(ResponseValue::stream(response)),
            400u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Gets the current ticket price

Get the current ticket price

Sends a `GET` request to `/api/v4/network/price`

*/
    pub async fn price<'a>(
        &'a self,
    ) -> Result<ResponseValue<types::TicketPriceResponse>, Error<()>> {
        let url = format!("{}/api/v4/network/price", self.baseurl,);
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .get(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "price",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Gets the current minimum incoming ticket winning probability defined by the network

Get the current minimum incoming ticket winning probability defined by the network

Sends a `GET` request to `/api/v4/network/probability`

*/
    pub async fn probability<'a>(
        &'a self,
    ) -> Result<ResponseValue<types::TicketProbabilityResponse>, Error<()>> {
        let url = format!("{}/api/v4/network/probability", self.baseurl,);
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .get(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "probability",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Get the configuration of the running node

Get the configuration of the running node

Sends a `GET` request to `/api/v4/node/configuration`

*/
    pub async fn configuration<'a>(
        &'a self,
    ) -> Result<
        ResponseValue<
            ::std::collections::HashMap<::std::string::String, ::std::string::String>,
        >,
        Error<()>,
    > {
        let url = format!("{}/api/v4/node/configuration", self.baseurl,);
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .get(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "configuration",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**List all known entry nodes with multiaddrs and eligibility

List all known entry nodes with multiaddrs and eligibility

Sends a `GET` request to `/api/v4/node/entry-nodes`

*/
    pub async fn entry_nodes<'a>(
        &'a self,
    ) -> Result<
        ResponseValue<
            ::std::collections::HashMap<::std::string::String, types::EntryNode>,
        >,
        Error<()>,
    > {
        let url = format!("{}/api/v4/node/entry-nodes", self.baseurl,);
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .get(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "entry_nodes",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Get information about this HOPR Node

Get information about this HOPR Node

Sends a `GET` request to `/api/v4/node/info`

*/
    pub async fn info<'a>(
        &'a self,
    ) -> Result<ResponseValue<types::NodeInfoResponse>, Error<()>> {
        let url = format!("{}/api/v4/node/info", self.baseurl,);
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .get(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "info",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Lists information for `connected peers` and `announced peers`

Lists information for connected and announced peers

Sends a `GET` request to `/api/v4/node/peers`

Arguments:
- `score`: Minimum peer quality to be included in the response.
*/
    pub async fn peers<'a>(
        &'a self,
        score: Option<f64>,
    ) -> Result<ResponseValue<types::NodePeersResponse>, Error<()>> {
        let url = format!("{}/api/v4/node/peers", self.baseurl,);
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .get(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .query(&progenitor_client::QueryParam::new("score", &score))
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "peers",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Get the release version of the running node

Get the release version of the running node

Sends a `GET` request to `/api/v4/node/version`

*/
    pub async fn version<'a>(
        &'a self,
    ) -> Result<ResponseValue<types::NodeVersionResponse>, Error<()>> {
        let url = format!("{}/api/v4/node/version", self.baseurl,);
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .get(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "version",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Returns transport-related information about the given peer

This includes the peer ids that the given peer has `announced` on-chain
and peer ids that are actually `observed` by the transport layer.

Sends a `GET` request to `/api/v4/peers/{destination}`

Arguments:
- `destination`: Address of the requested peer
*/
    pub async fn show_peer_info<'a>(
        &'a self,
        destination: &'a str,
    ) -> Result<ResponseValue<types::NodePeerInfoResponse>, Error<()>> {
        let url = format!(
            "{}/api/v4/peers/{}", self.baseurl, encode_path(& destination.to_string()),
        );
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .get(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "show_peer_info",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Directly pings the given peer

Directly ping the given peer

Sends a `POST` request to `/api/v4/peers/{destination}/ping`

Arguments:
- `destination`: Address of the requested peer
*/
    pub async fn ping_peer<'a>(
        &'a self,
        destination: &'a str,
    ) -> Result<ResponseValue<types::PingResponse>, Error<()>> {
        let url = format!(
            "{}/api/v4/peers/{}/ping", self.baseurl, encode_path(& destination
            .to_string()),
        );
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .post(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "ping_peer",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Get packet statistics for a specific connected peer

Get packet statistics for a specific connected peer

Sends a `GET` request to `/api/v4/peers/{destination}/stats`

Arguments:
- `destination`: Address of the requested peer
*/
    pub async fn peer_stats<'a>(
        &'a self,
        destination: &'a str,
    ) -> Result<ResponseValue<types::PeerPacketStatsResponse>, Error<()>> {
        let url = format!(
            "{}/api/v4/peers/{}/stats", self.baseurl, encode_path(& destination
            .to_string()),
        );
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .get(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "peer_stats",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Gets configuration of an existing active session.

Sends a `GET` request to `/api/v4/session/config/{id}`

Arguments:
- `id`: Session ID
*/
    pub async fn session_config<'a>(
        &'a self,
        id: &'a str,
    ) -> Result<ResponseValue<types::SessionConfig>, Error<()>> {
        let url = format!(
            "{}/api/v4/session/config/{}", self.baseurl, encode_path(& id.to_string()),
        );
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .get(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "session_config",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Updates configuration of an existing active session.

Sends a `POST` request to `/api/v4/session/config/{id}`

Arguments:
- `id`: Session ID
- `body`: Allows updating of several parameters of an existing active session.
*/
    pub async fn adjust_session<'a>(
        &'a self,
        id: &'a str,
        body: &'a types::SessionConfig,
    ) -> Result<ResponseValue<ByteStream>, Error<types::ApiError>> {
        let url = format!(
            "{}/api/v4/session/config/{}", self.baseurl, encode_path(& id.to_string()),
        );
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self.client.post(url).json(&body).headers(header_map).build()?;
        let info = OperationInfo {
            operation_id: "adjust_session",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200..=299 => Ok(ResponseValue::stream(response)),
            400u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Gets stats for an existing active session.

Sends a `GET` request to `/api/v4/session/stats/{id}`

Arguments:
- `id`: Session ID
*/
    pub async fn session_stats<'a>(
        &'a self,
        id: &'a str,
    ) -> Result<ResponseValue<types::SessionStatsResponse>, Error<()>> {
        let url = format!(
            "{}/api/v4/session/stats/{}", self.baseurl, encode_path(& id.to_string()),
        );
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .get(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "session_stats",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Lists existing Session listeners for the given IP protocol

Lists existing Session listeners for the given IP protocol.

Sends a `GET` request to `/api/v4/session/{protocol}`

Arguments:
- `protocol`: IP transport protocol
*/
    pub async fn list_clients<'a>(
        &'a self,
        protocol: &'a str,
    ) -> Result<
        ResponseValue<::std::vec::Vec<types::SessionClientResponse>>,
        Error<()>,
    > {
        let url = format!(
            "{}/api/v4/session/{}", self.baseurl, encode_path(& protocol.to_string()),
        );
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .get(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "list_clients",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Creates a new client session returning the given session listening host and port over TCP or UDP.
If no listening port is given in the request, the socket will be bound to a random free
port and returned in the response.
Different capabilities can be configured for the session, such as data segmentation or
retransmission

Creates a new client HOPR session that will start listening on a dedicated port. Once the port is bound, it is possible to use the socket for bidirectional read and write communication.

Sends a `POST` request to `/api/v4/session/{protocol}`

Arguments:
- `protocol`: IP transport protocol
- `body`: Creates a new client HOPR session that will start listening on a dedicated port. Once the port is bound, it is possible to use the socket for bidirectional read and write communication.
*/
    pub async fn create_client<'a>(
        &'a self,
        protocol: &'a str,
        body: &'a types::SessionClientRequest,
    ) -> Result<ResponseValue<types::SessionClientResponse>, Error<()>> {
        let url = format!(
            "{}/api/v4/session/{}", self.baseurl, encode_path(& protocol.to_string()),
        );
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .post(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .json(&body)
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "create_client",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Closes an existing Session listener.
The listener must've been previously created and bound for the given IP protocol.
Once a listener is closed, no more socket connections can be made to it.
If the passed port number is 0, listeners on all ports of the given listening IP and protocol
will be closed

Closes an existing Session listener.

Sends a `DELETE` request to `/api/v4/session/{protocol}/{ip}/{port}`

Arguments:
- `protocol`: IP transport protocol
- `ip`: Listening IP address of the Session.
- `port`: Session port used for the listener.
*/
    pub async fn close_client<'a>(
        &'a self,
        protocol: types::IpProtocol,
        ip: &'a str,
        port: i32,
    ) -> Result<ResponseValue<ByteStream>, Error<types::ApiError>> {
        let url = format!(
            "{}/api/v4/session/{}/{}/{}", self.baseurl, encode_path(& protocol
            .to_string()), encode_path(& ip.to_string()), encode_path(& port
            .to_string()),
        );
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self.client.delete(url).headers(header_map).build()?;
        let info = OperationInfo {
            operation_id: "close_client",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200..=299 => Ok(ResponseValue::stream(response)),
            400u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Endpoint is deprecated and will be removed in the future. Returns an empty array

(deprecated) Returns an empty array.

Sends a `GET` request to `/api/v4/tickets`

*/
    pub async fn show_all_tickets<'a>(
        &'a self,
    ) -> Result<ResponseValue<::std::vec::Vec<types::ChannelTicket>>, Error<()>> {
        let url = format!("{}/api/v4/tickets", self.baseurl,);
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .get(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "show_all_tickets",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Starts redeeming of all tickets in all channels

Starts redeeming of all tickets in all channels.

Sends a `POST` request to `/api/v4/tickets/redeem`

*/
    pub async fn redeem_all_tickets<'a>(
        &'a self,
    ) -> Result<ResponseValue<ByteStream>, Error<types::ApiError>> {
        let url = format!("{}/api/v4/tickets/redeem", self.baseurl,);
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self.client.post(url).headers(header_map).build()?;
        let info = OperationInfo {
            operation_id: "redeem_all_tickets",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200..=299 => Ok(ResponseValue::stream(response)),
            401u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Returns current complete statistics on tickets

Returns current complete statistics on tickets.

Sends a `GET` request to `/api/v4/tickets/statistics`

*/
    pub async fn show_ticket_statistics<'a>(
        &'a self,
    ) -> Result<ResponseValue<types::NodeTicketStatisticsResponse>, Error<()>> {
        let url = format!("{}/api/v4/tickets/statistics", self.baseurl,);
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .get(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "show_ticket_statistics",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Resets the ticket metrics

Resets the ticket metrics.

Sends a `DELETE` request to `/api/v4/tickets/statistics`

*/
    pub async fn reset_ticket_statistics<'a>(
        &'a self,
    ) -> Result<ResponseValue<ByteStream>, Error<types::ApiError>> {
        let url = format!("{}/api/v4/tickets/statistics", self.baseurl,);
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self.client.delete(url).headers(header_map).build()?;
        let info = OperationInfo {
            operation_id: "reset_ticket_statistics",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200..=299 => Ok(ResponseValue::stream(response)),
            401u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Check whether the node is eligible in the network

Check whether the node is eligible in the network

Sends a `GET` request to `/eligiblez`

*/
    pub async fn eligiblez<'a>(&'a self) -> Result<ResponseValue<()>, Error<()>> {
        let url = format!("{}/eligiblez", self.baseurl,);
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self.client.get(url).headers(header_map).build()?;
        let info = OperationInfo {
            operation_id: "eligiblez",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => Ok(ResponseValue::empty(response)),
            412u16 => Err(Error::ErrorResponse(ResponseValue::empty(response))),
            500u16 => Err(Error::ErrorResponse(ResponseValue::empty(response))),
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Check whether the node is **healthy**

Check whether the node is healthy

Sends a `GET` request to `/healthyz`

*/
    pub async fn healthyz<'a>(&'a self) -> Result<ResponseValue<()>, Error<()>> {
        let url = format!("{}/healthyz", self.baseurl,);
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self.client.get(url).headers(header_map).build()?;
        let info = OperationInfo {
            operation_id: "healthyz",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => Ok(ResponseValue::empty(response)),
            412u16 => Err(Error::ErrorResponse(ResponseValue::empty(response))),
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Retrieve Prometheus metrics from the running node

Retrieve Prometheus metrics from the running node

Sends a `GET` request to `/metrics`

*/
    pub async fn metrics<'a>(&'a self) -> Result<ResponseValue<ByteStream>, Error<()>> {
        let url = format!("{}/metrics", self.baseurl,);
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self.client.get(url).headers(header_map).build()?;
        let info = OperationInfo {
            operation_id: "metrics",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => Ok(ResponseValue::stream(response)),
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Check whether the node is **ready** to accept connections

Check whether the node is ready to accept connections

Sends a `GET` request to `/readyz`

*/
    pub async fn readyz<'a>(&'a self) -> Result<ResponseValue<()>, Error<()>> {
        let url = format!("{}/readyz", self.baseurl,);
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self.client.get(url).headers(header_map).build()?;
        let info = OperationInfo {
            operation_id: "readyz",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => Ok(ResponseValue::empty(response)),
            412u16 => Err(Error::ErrorResponse(ResponseValue::empty(response))),
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Check whether the node is started

Check whether the node is started

Sends a `GET` request to `/startedz`

*/
    pub async fn startedz<'a>(&'a self) -> Result<ResponseValue<()>, Error<()>> {
        let url = format!("{}/startedz", self.baseurl,);
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self.client.get(url).headers(header_map).build()?;
        let info = OperationInfo {
            operation_id: "startedz",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => Ok(ResponseValue::empty(response)),
            412u16 => Err(Error::ErrorResponse(ResponseValue::empty(response))),
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
}
/// Items consumers will typically use such as the Client.
pub mod prelude {
    #[allow(unused_imports)]
    pub use super::Client;
}
