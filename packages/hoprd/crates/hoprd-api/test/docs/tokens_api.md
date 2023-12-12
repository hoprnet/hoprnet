# tokens_api

All URIs are relative to */api/v3*

Method | HTTP request | Description
------------- | ------------- | -------------
**tokensCreate**](tokens_api.md#tokensCreate) | **POST** /tokens/ | 
**tokensDelete**](tokens_api.md#tokensDelete) | **DELETE** /tokens/{id} | 
**tokensGetToken**](tokens_api.md#tokensGetToken) | **GET** /token | 


# **tokensCreate**
> models::TokensCreate201Response tokensCreate(ctx, ctx, optional)


Create a new authentication token based on the given information. The new token is returned as part of the response body and must be stored by the client. It cannot be read again in cleartext and is lost, if the client loses the token. An authentication has a lifetime. It can be unbound, meaning it will not expire. Or it has a limited lifetime after which it expires. The requested limited lifetime is requested by the client in seconds.

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **tokens_create_request** | [**TokensCreateRequest**](TokensCreateRequest.md)|  | 

### Return type

[**models::TokensCreate201Response**](tokensCreate_201_response.md)

### Authorization

[passwordScheme](../README.md#passwordScheme), [keyScheme](../README.md#keyScheme)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **tokensDelete**
> tokensDelete(ctx, ctx, id)


Deletes a token. Can only be done before the lifetime expired. After the lifetime expired the token is automatically deleted.

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **id** | **String**| ID of the token which shall be deleted. | 

### Return type

 (empty response body)

### Authorization

[passwordScheme](../README.md#passwordScheme), [keyScheme](../README.md#keyScheme)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **tokensGetToken**
> models::Token tokensGetToken(ctx, ctx, )


Get the full token information for the token used in authentication.

### Required Parameters
This endpoint does not need any parameter.

### Return type

[**models::Token**](Token.md)

### Authorization

[passwordScheme](../README.md#passwordScheme), [keyScheme](../README.md#keyScheme)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

