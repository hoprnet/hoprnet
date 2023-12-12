# peer_info_api

All URIs are relative to */api/v3*

Method | HTTP request | Description
------------- | ------------- | -------------
**peerInfoGetPeerInfo**](peer_info_api.md#peerInfoGetPeerInfo) | **GET** /peers/{peerid}/ | 


# **peerInfoGetPeerInfo**
> models::PeerInfoGetPeerInfo200Response peerInfoGetPeerInfo(ctx, ctx, peerid)


Get information about this peer.

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **peerid** | **String**|  | 

### Return type

[**models::PeerInfoGetPeerInfo200Response**](peerInfoGetPeerInfo_200_response.md)

### Authorization

[passwordScheme](../README.md#passwordScheme), [keyScheme](../README.md#keyScheme)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

