# settings_api

All URIs are relative to */api/v3*

Method | HTTP request | Description
------------- | ------------- | -------------
**settingsGetSettings**](settings_api.md#settingsGetSettings) | **GET** /settings/ | 
**settingsSetSetting**](settings_api.md#settingsSetSetting) | **PUT** /settings/{setting} | 


# **settingsGetSettings**
> models::Settings settingsGetSettings(ctx, ctx, )


Get all of the node's settings.

### Required Parameters
This endpoint does not need any parameter.

### Return type

[**models::Settings**](Settings.md)

### Authorization

[passwordScheme](../README.md#passwordScheme), [keyScheme](../README.md#keyScheme)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **settingsSetSetting**
> settingsSetSetting(ctx, ctx, setting, optional)


Change this node's setting value. Check Settings schema to learn more about each setting and the type of value it expects.

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **setting** | **String**|  | 
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **setting** | **String**|  | 
 **settings_set_setting_request** | [**SettingsSetSettingRequest**](SettingsSetSettingRequest.md)|  | 

### Return type

 (empty response body)

### Authorization

[passwordScheme](../README.md#passwordScheme), [keyScheme](../README.md#keyScheme)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

