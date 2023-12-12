# node_api

All URIs are relative to _/api/v3_

| Method                                                | HTTP request             | Description |
| ----------------------------------------------------- | ------------------------ | ----------- |
| **nodeGetEntryNodes**](node_api.md#nodeGetEntryNodes) | **GET** /node/entryNodes |
| **nodeGetInfo**](node_api.md#nodeGetInfo)             | **GET** /node/info       |
| **nodeGetMetrics**](node_api.md#nodeGetMetrics)       | **GET** /node/metrics    |
| **nodeGetPeers**](node_api.md#nodeGetPeers)           | **GET** /node/peers      |
| **nodeGetVersion**](node_api.md#nodeGetVersion)       | **GET** /node/version    |

# **nodeGetEntryNodes**

> std::collections::HashMap<String, models::NodeGetEntryNodes200ResponseValue> nodeGetEntryNodes(ctx, ctx, )

List all known entry nodes and their multiaddrs and their eligibility state

### Required Parameters

This endpoint does not need any parameter.

### Return type

[**std::collections::HashMap<String, models::NodeGetEntryNodes200ResponseValue>**](nodeGetEntryNodes_200_response_value.md)

### Authorization

[passwordScheme](../README.md#passwordScheme), [keyScheme](../README.md#keyScheme)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **nodeGetInfo**

> models::NodeGetInfo200Response nodeGetInfo(ctx, ctx, )

Information about the HOPR Node, including any options it started with. See the schema of the response to get more information on each field.

### Required Parameters

This endpoint does not need any parameter.

### Return type

[**models::NodeGetInfo200Response**](nodeGetInfo_200_response.md)

### Authorization

[passwordScheme](../README.md#passwordScheme), [keyScheme](../README.md#keyScheme)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **nodeGetMetrics**

> String nodeGetMetrics(ctx, ctx, )

Retrieve Prometheus metrics from the running node.

### Required Parameters

This endpoint does not need any parameter.

### Return type

[**String**](string.md)

### Authorization

[passwordScheme](../README.md#passwordScheme), [keyScheme](../README.md#keyScheme)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, text/plain; version=0.0.4

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **nodeGetPeers**

> models::NodeGetPeers200Response nodeGetPeers(ctx, ctx, optional)

Lists information for `connected peers` and `announced peers`. Connected peers are nodes which are connected to the node while announced peers are nodes which have announced to the network. Optionally, you can pass `quality` parameter which would filter out peers with lower quality to the one specified.

### Required Parameters

| Name         | Type                       | Description                           | Notes                    |
| ------------ | -------------------------- | ------------------------------------- | ------------------------ |
| **ctx**      | **context.Context**        | context containing the authentication | nil if no authentication |
| **ctx**      | **context.Context**        | context containing the authentication | nil if no authentication |
| **optional** | **map[string]interface{}** | optional parameters                   | nil if no parameters     |

### Optional Parameters

Optional parameters are passed through a map[string]interface{}.

| Name        | Type    | Description                                                                                                     | Notes |
| ----------- | ------- | --------------------------------------------------------------------------------------------------------------- | ----- |
| **quality** | **f64** | When quality is passed, the response will only include peers with higher or equal quality to the one specified. |

### Return type

[**models::NodeGetPeers200Response**](nodeGetPeers_200_response.md)

### Authorization

[passwordScheme](../README.md#passwordScheme), [keyScheme](../README.md#keyScheme)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **nodeGetVersion**

> String nodeGetVersion(ctx, ctx, )

Get release version of the running node.

### Required Parameters

This endpoint does not need any parameter.

### Return type

[**String**](string.md)

### Authorization

[passwordScheme](../README.md#passwordScheme), [keyScheme](../README.md#keyScheme)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
