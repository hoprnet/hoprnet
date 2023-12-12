# Ticket

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**channel_id** | [***serde_json::Value**](.md) | The unique identifier of a unidirectional HOPR channel. | [optional] [default to None]
**amount** | **String** | The ticket's value in HOPR. Only relevant if ticket is a win. | [optional] [default to None]
**index** | **String** | Each ticket is labeled by an ongoing serial number named ticket index i and its current value is stored in the smart contract. | [optional] [default to None]
**index_offset** | **String** | Offset by which the on-chain stored ticket index gets increased when redeeming the ticket. Used to aggregate tickets. | [optional] [default to None]
**channel_epoch** | **String** | Payment channels might run through multiple open and close sequences, this epoch tracks the sequence. | [optional] [default to None]
**win_prob** | **String** | The ticket's winning probability, going from 0.0 to 1.0 where 0.0 ~= 0% winning probability and 1.0 equals 100% winning probability. | [optional] [default to None]
**signature** | **String** | Signature from requested message. | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


