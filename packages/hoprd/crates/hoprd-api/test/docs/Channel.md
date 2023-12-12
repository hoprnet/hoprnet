# Channel

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**r#type** | **String** | Channel can be either incomming or outgoing. Incomming means that other node can send messages using this node as relay. Outgoing means that this node can use other node to send message as realy. | [optional] [default to None]
**id** | [***serde_json::Value**](.md) | The unique identifier of a unidirectional HOPR channel. | [optional] [default to None]
**peer_id** | **String** | HOPR account address, also called a PeerId. Used to send / receive messages, open / close payment channels. | [optional] [default to None]
**status** | [***models::ChannelStatus**](ChannelStatus.md) |  | [optional] [default to None]
**balance** | **String** | Amount of HOPR tokens in the smallest unit. Used for funding payment channels. | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


