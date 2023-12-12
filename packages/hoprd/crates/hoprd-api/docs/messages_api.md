# messages_api

All URIs are relative to _/api/v3_

| Method                                                              | HTTP request                | Description |
| ------------------------------------------------------------------- | --------------------------- | ----------- |
| **messagesDeleteMessages**](messages_api.md#messagesDeleteMessages) | **DELETE** /messages/       |
| **messagesGetSize**](messages_api.md#messagesGetSize)               | **GET** /messages/size      |
| **messagesPopAllMessage**](messages_api.md#messagesPopAllMessage)   | **POST** /messages/pop-all  |
| **messagesPopMessage**](messages_api.md#messagesPopMessage)         | **POST** /messages/pop      |
| **messagesSendMessage**](messages_api.md#messagesSendMessage)       | **POST** /messages/         |
| **messagesWebsocket**](messages_api.md#messagesWebsocket)           | **GET** /messages/websocket |

# **messagesDeleteMessages**

> messagesDeleteMessages(ctx, ctx, tag)

Delete messages from nodes message inbox. Does not return any data.

### Required Parameters

| Name    | Type                | Description                           | Notes                    |
| ------- | ------------------- | ------------------------------------- | ------------------------ |
| **ctx** | **context.Context** | context containing the authentication | nil if no authentication |
| **ctx** | **context.Context** | context containing the authentication | nil if no authentication |
| **tag** | **i32**             | Tag used to filter target messages.   |

### Return type

(empty response body)

### Authorization

[passwordScheme](../README.md#passwordScheme), [keyScheme](../README.md#keyScheme)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **messagesGetSize**

> models::MessagesGetSize200Response messagesGetSize(ctx, ctx, tag)

Get size of filtered message inbox.

### Required Parameters

| Name    | Type                | Description                           | Notes                    |
| ------- | ------------------- | ------------------------------------- | ------------------------ |
| **ctx** | **context.Context** | context containing the authentication | nil if no authentication |
| **ctx** | **context.Context** | context containing the authentication | nil if no authentication |
| **tag** | **i32**             | Tag used to filter target messages.   |

### Return type

[**models::MessagesGetSize200Response**](messagesGetSize_200_response.md)

### Authorization

[passwordScheme](../README.md#passwordScheme), [keyScheme](../README.md#keyScheme)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **messagesPopAllMessage**

> models::MessagesPopAllMessage200Response messagesPopAllMessage(ctx, ctx, optional)

Get list of messages currently present in the nodes message inbox. The messages are removed from the inbox.

### Required Parameters

| Name         | Type                       | Description                           | Notes                    |
| ------------ | -------------------------- | ------------------------------------- | ------------------------ |
| **ctx**      | **context.Context**        | context containing the authentication | nil if no authentication |
| **ctx**      | **context.Context**        | context containing the authentication | nil if no authentication |
| **optional** | **map[string]interface{}** | optional parameters                   | nil if no parameters     |

### Optional Parameters

Optional parameters are passed through a map[string]interface{}.

| Name                                 | Type                                                                | Description | Notes |
| ------------------------------------ | ------------------------------------------------------------------- | ----------- | ----- |
| **messages_pop_all_message_request** | [**MessagesPopAllMessageRequest**](MessagesPopAllMessageRequest.md) |             |

### Return type

[**models::MessagesPopAllMessage200Response**](messagesPopAllMessage_200_response.md)

### Authorization

[passwordScheme](../README.md#passwordScheme), [keyScheme](../README.md#keyScheme)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **messagesPopMessage**

> models::ReceivedMessage messagesPopMessage(ctx, ctx, optional)

Get oldest message currently present in the nodes message inbox. The message is removed from the inbox.

### Required Parameters

| Name         | Type                       | Description                           | Notes                    |
| ------------ | -------------------------- | ------------------------------------- | ------------------------ |
| **ctx**      | **context.Context**        | context containing the authentication | nil if no authentication |
| **ctx**      | **context.Context**        | context containing the authentication | nil if no authentication |
| **optional** | **map[string]interface{}** | optional parameters                   | nil if no parameters     |

### Optional Parameters

Optional parameters are passed through a map[string]interface{}.

| Name                                 | Type                                                                | Description | Notes |
| ------------------------------------ | ------------------------------------------------------------------- | ----------- | ----- |
| **messages_pop_all_message_request** | [**MessagesPopAllMessageRequest**](MessagesPopAllMessageRequest.md) |             |

### Return type

[**models::ReceivedMessage**](ReceivedMessage.md)

### Authorization

[passwordScheme](../README.md#passwordScheme), [keyScheme](../README.md#keyScheme)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **messagesSendMessage**

> String messagesSendMessage(ctx, ctx, optional)

Send a message to another peer using a given path (list of node addresses that should relay our message through network). If no path is given, HOPR will attempt to find a path.

### Required Parameters

| Name         | Type                       | Description                           | Notes                    |
| ------------ | -------------------------- | ------------------------------------- | ------------------------ |
| **ctx**      | **context.Context**        | context containing the authentication | nil if no authentication |
| **ctx**      | **context.Context**        | context containing the authentication | nil if no authentication |
| **optional** | **map[string]interface{}** | optional parameters                   | nil if no parameters     |

### Optional Parameters

Optional parameters are passed through a map[string]interface{}.

| Name                              | Type                                                            | Description | Notes |
| --------------------------------- | --------------------------------------------------------------- | ----------- | ----- |
| **messages_send_message_request** | [**MessagesSendMessageRequest**](MessagesSendMessageRequest.md) |             |

### Return type

[**String**](string.md)

### Authorization

[passwordScheme](../README.md#passwordScheme), [keyScheme](../README.md#keyScheme)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **messagesWebsocket**

> String messagesWebsocket(ctx, ctx, )

This is a websocket endpoint which exposes a subset of message functions. Incoming messages from other nodes are sent to the websocket client. A client may also send message by sending the following data: { cmd: \"sendmsg\", args: { peerId: \"SOME_PEER_ID\", path: [], hops: 1} } The command arguments follow the same semantics as in the dedicated API endpoint for sending messages. The following messages may be sent by the server over the Websocket connection: { type: \"message\", tag: 12, body: \"my example message\" } { type: \"message-ack\", id: \"some challenge id\" } { type: \"message-ack-challenge\", id: \"some challenge id\" } Authentication (if enabled) is done via either passing an `apiToken` parameter in the url or cookie `X-Auth-Token`. Connect to the endpoint by using a WS client. No preview available. Example: `ws://127.0.0.1:3001/api/v2/messages/websocket/?apiToken=myApiToken`

### Required Parameters

This endpoint does not need any parameter.

### Return type

[**String**](string.md)

### Authorization

[passwordScheme](../README.md#passwordScheme), [keyScheme](../README.md#keyScheme)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, application/text

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
