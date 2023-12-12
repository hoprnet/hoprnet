# NodeGetPeers200ResponseConnectedInner

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**peer_id** | **String** | HOPR account address, also called a PeerId. Used to send / receive messages, open / close payment channels. | [optional] [default to None]
**peer_address** | **String** | Blockchain-native account address. Can be funded from external wallets (starts with **0x...**). It **can't be used** internally to send / receive messages, open / close payment channels. | [optional] [default to None]
**multi_addr** | **String** | A multi address is a composable and future-proof network address, usually announced by Public HOPR nodes. | [optional] [default to None]
**heartbeats** | [***models::NodeGetPeers200ResponseConnectedInnerHeartbeats**](nodeGetPeers_200_response_connected_inner_heartbeats.md) |  | [optional] [default to None]
**last_seen** | **f64** | Timestamp on when the node was last seen (in milliseconds) | [optional] [default to None]
**last_seen_latency** | **f64** | Latency recorded the last time a node was measured when seen (in milliseconds) | [optional] [default to None]
**quality** | **f64** | A float between 0 (completely unreliable) and 1 (completely reliable) estimating the quality of service of a peer's network connection | [optional] [default to None]
**backoff** | **f64** |  | [optional] [default to None]
**is_new** | **bool** | True if the node is new (no heartbeats sent yet). | [optional] [default to None]
**reported_version** | **String** | HOPR protocol version as determined from the successful ping in the Major.Minor.Patch format or \"unknown\" | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


