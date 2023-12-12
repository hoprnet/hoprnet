# ChannelTopology

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**channel_id** | [***serde_json::Value**](.md) | The unique identifier of a unidirectional HOPR channel. | [optional] [default to None]
**source_peer_id** | **String** | HOPR account address, also called a PeerId. Used to send / receive messages, open / close payment channels. | [optional] [default to None]
**destination_peer_id** | **String** | HOPR account address, also called a PeerId. Used to send / receive messages, open / close payment channels. | [optional] [default to None]
**source_address** | **String** | Blockchain-native account address. Can be funded from external wallets (starts with **0x...**). It **can't be used** internally to send / receive messages, open / close payment channels. | [optional] [default to None]
**destination_address** | **String** | Blockchain-native account address. Can be funded from external wallets (starts with **0x...**). It **can't be used** internally to send / receive messages, open / close payment channels. | [optional] [default to None]
**balance** | **String** | Amount of HOPR tokens in the smallest unit. Used for funding payment channels. | [optional] [default to None]
**status** | [***models::ChannelStatus**](ChannelStatus.md) |  | [optional] [default to None]
**ticket_index** | **String** | Each ticket is labeled by an ongoing serial number named ticket index i and its current value is stored in the smart contract. | [optional] [default to None]
**channel_epoch** | **String** | Payment channels might run through multiple open and close sequences, this epoch tracks the sequence. | [optional] [default to None]
**closure_time** | **String** | Time when the channel can be closed | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


