# AccountGetAddresses200Response

## Properties

| Name       | Type       | Description                                                                                                                                                                                | Notes                        |
| ---------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ---------------------------- |
| **native** | **String** | Blockchain-native account address. Can be funded from external wallets (starts with **0x...**). It **can't be used** internally to send / receive messages, open / close payment channels. | [optional] [default to None] |
| **hopr**   | **String** | HOPR account address, also called a PeerId. Used to send / receive messages, open / close payment channels.                                                                                | [optional] [default to None] |

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
