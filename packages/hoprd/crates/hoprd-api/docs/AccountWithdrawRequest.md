# AccountWithdrawRequest

## Properties

| Name                 | Type                                  | Description                                                                                                                                                                                | Notes |
| -------------------- | ------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ----- |
| **currency**         | [**\*models::Currency**](Currency.md) |                                                                                                                                                                                            |
| **amount**           | **String**                            | Amount to withdraw in the currency's smallest unit.                                                                                                                                        |
| **ethereum_address** | **String**                            | Blockchain-native account address. Can be funded from external wallets (starts with **0x...**). It **can't be used** internally to send / receive messages, open / close payment channels. |

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
