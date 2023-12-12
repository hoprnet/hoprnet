# peers_api

All URIs are relative to _/api/v3_

| Method                                         | HTTP request                  | Description |
| ---------------------------------------------- | ----------------------------- | ----------- |
| **peersPingPeer**](peers_api.md#peersPingPeer) | **POST** /peers/{peerid}/ping |

# **peersPingPeer**

> models::PeersPingPeer200Response peersPingPeer(ctx, ctx, peerid)

Pings another node to check its availability.

### Required Parameters

| Name       | Type                | Description                           | Notes                    |
| ---------- | ------------------- | ------------------------------------- | ------------------------ |
| **ctx**    | **context.Context** | context containing the authentication | nil if no authentication |
| **ctx**    | **context.Context** | context containing the authentication | nil if no authentication |
| **peerid** | **String**          | Peer id that should be pinged         |

### Return type

[**models::PeersPingPeer200Response**](peersPingPeer_200_response.md)

### Authorization

[passwordScheme](../README.md#passwordScheme), [keyScheme](../README.md#keyScheme)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
