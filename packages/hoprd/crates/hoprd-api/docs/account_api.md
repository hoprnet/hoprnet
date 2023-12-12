# account_api

All URIs are relative to _/api/v3_

| Method                                                       | HTTP request               | Description |
| ------------------------------------------------------------ | -------------------------- | ----------- |
| **accountGetAddress**](account_api.md#accountGetAddress)     | **GET** /account/address   |
| **accountGetAddresses**](account_api.md#accountGetAddresses) | **GET** /account/addresses |
| **accountGetBalances**](account_api.md#accountGetBalances)   | **GET** /account/balances  |
| **accountWithdraw**](account_api.md#accountWithdraw)         | **POST** /account/withdraw |

# **accountGetAddress**

> models::AccountGetAddresses200Response accountGetAddress(ctx, ctx, )

Get node's HOPR and native addresses. HOPR address is also called PeerId and can be used by other node owner to interact with this node.

### Required Parameters

This endpoint does not need any parameter.

### Return type

[**models::AccountGetAddresses200Response**](accountGetAddresses_200_response.md)

### Authorization

[passwordScheme](../README.md#passwordScheme), [keyScheme](../README.md#keyScheme)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **accountGetAddresses**

> models::AccountGetAddresses200Response accountGetAddresses(ctx, ctx, )

Get node's HOPR and native addresses. HOPR address is also called PeerId and can be used by other node owner to interact with this node.

### Required Parameters

This endpoint does not need any parameter.

### Return type

[**models::AccountGetAddresses200Response**](accountGetAddresses_200_response.md)

### Authorization

[passwordScheme](../README.md#passwordScheme), [keyScheme](../README.md#keyScheme)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **accountGetBalances**

> models::AccountGetBalances200Response accountGetBalances(ctx, ctx, )

Get node's and associated Safe's HOPR and native balances as well as the allowance for HOPR tokens to be drawn by HoprChannels from Safe. HOPR tokens from the Safe balance is used to fund payment channels between this node and other nodes on the network. NATIVE balance of the Node is used to pay for the gas fees for the blockchain.

### Required Parameters

This endpoint does not need any parameter.

### Return type

[**models::AccountGetBalances200Response**](accountGetBalances_200_response.md)

### Authorization

[passwordScheme](../README.md#passwordScheme), [keyScheme](../README.md#keyScheme)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **accountWithdraw**

> models::AccountWithdraw200Response accountWithdraw(ctx, ctx, optional)

Withdraw funds from this node to your ethereum wallet address. You can choose whitch currency you want to withdraw, NATIVE or HOPR.

### Required Parameters

| Name         | Type                       | Description                           | Notes                    |
| ------------ | -------------------------- | ------------------------------------- | ------------------------ |
| **ctx**      | **context.Context**        | context containing the authentication | nil if no authentication |
| **ctx**      | **context.Context**        | context containing the authentication | nil if no authentication |
| **optional** | **map[string]interface{}** | optional parameters                   | nil if no parameters     |

### Optional Parameters

Optional parameters are passed through a map[string]interface{}.

| Name                         | Type                                                    | Description | Notes |
| ---------------------------- | ------------------------------------------------------- | ----------- | ----- |
| **account_withdraw_request** | [**AccountWithdrawRequest**](AccountWithdrawRequest.md) |             |

### Return type

[**models::AccountWithdraw200Response**](accountWithdraw_200_response.md)

### Authorization

[passwordScheme](../README.md#passwordScheme), [keyScheme](../README.md#keyScheme)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
