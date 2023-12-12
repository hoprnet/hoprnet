# aliases_api

All URIs are relative to _/api/v3_

| Method                                                     | HTTP request                | Description |
| ---------------------------------------------------------- | --------------------------- | ----------- |
| **aliasesGetAlias**](aliases_api.md#aliasesGetAlias)       | **GET** /aliases/{alias}    |
| **aliasesGetAliases**](aliases_api.md#aliasesGetAliases)   | **GET** /aliases/           |
| **aliasesRemoveAlias**](aliases_api.md#aliasesRemoveAlias) | **DELETE** /aliases/{alias} |
| **aliasesSetAlias**](aliases_api.md#aliasesSetAlias)       | **POST** /aliases/          |

# **aliasesGetAlias**

> models::AliasesGetAlias200Response aliasesGetAlias(ctx, ctx, alias)

Get the PeerId (Hopr address) that have this alias assigned to it.

### Required Parameters

| Name      | Type                | Description                                       | Notes                    |
| --------- | ------------------- | ------------------------------------------------- | ------------------------ |
| **ctx**   | **context.Context** | context containing the authentication             | nil if no authentication |
| **ctx**   | **context.Context** | context containing the authentication             | nil if no authentication |
| **alias** | **String**          | Alias that we previously assigned to some PeerId. |

### Return type

[**models::AliasesGetAlias200Response**](aliasesGetAlias_200_response.md)

### Authorization

[passwordScheme](../README.md#passwordScheme), [keyScheme](../README.md#keyScheme)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **aliasesGetAliases**

> models::AliasesGetAliases200Response aliasesGetAliases(ctx, ctx, )

Get all aliases you set previously and thier corresponding peer IDs.

### Required Parameters

This endpoint does not need any parameter.

### Return type

[**models::AliasesGetAliases200Response**](aliasesGetAliases_200_response.md)

### Authorization

[passwordScheme](../README.md#passwordScheme), [keyScheme](../README.md#keyScheme)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **aliasesRemoveAlias**

> aliasesRemoveAlias(ctx, ctx, alias)

Unassign an alias from a PeerId. You can always assign back alias to that PeerId using /aliases endpoint.

### Required Parameters

| Name      | Type                | Description                           | Notes                    |
| --------- | ------------------- | ------------------------------------- | ------------------------ |
| **ctx**   | **context.Context** | context containing the authentication | nil if no authentication |
| **ctx**   | **context.Context** | context containing the authentication | nil if no authentication |
| **alias** | **String**          | Alias that we want to remove.         |

### Return type

(empty response body)

### Authorization

[passwordScheme](../README.md#passwordScheme), [keyScheme](../README.md#keyScheme)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **aliasesSetAlias**

> aliasesSetAlias(ctx, ctx, optional)

Instead of using HOPR address, we can assign HOPR address to a specific name called alias. Give an address a more memorable alias and use it instead of Hopr address. Aliases are kept locally and are not saved or shared on the network.

### Required Parameters

| Name         | Type                       | Description                           | Notes                    |
| ------------ | -------------------------- | ------------------------------------- | ------------------------ |
| **ctx**      | **context.Context**        | context containing the authentication | nil if no authentication |
| **ctx**      | **context.Context**        | context containing the authentication | nil if no authentication |
| **optional** | **map[string]interface{}** | optional parameters                   | nil if no parameters     |

### Optional Parameters

Optional parameters are passed through a map[string]interface{}.

| Name                          | Type                                                    | Description | Notes |
| ----------------------------- | ------------------------------------------------------- | ----------- | ----- |
| **aliases_set_alias_request** | [**AliasesSetAliasRequest**](AliasesSetAliasRequest.md) |             |

### Return type

(empty response body)

### Authorization

[passwordScheme](../README.md#passwordScheme), [keyScheme](../README.md#keyScheme)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
