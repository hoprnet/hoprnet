# channels_api

All URIs are relative to */api/v3*

Method | HTTP request | Description
------------- | ------------- | -------------
**channelsAggregateTickets**](channels_api.md#channelsAggregateTickets) | **POST** /channels/{channelid}/tickets/aggregate | 
**channelsCloseChannel**](channels_api.md#channelsCloseChannel) | **DELETE** /channels/{channelid}/ | 
**channelsFundChannel**](channels_api.md#channelsFundChannel) | **POST** /channels/{channelid}/fund | 
**channelsGetChannel**](channels_api.md#channelsGetChannel) | **GET** /channels/{channelid}/ | 
**channelsGetChannels**](channels_api.md#channelsGetChannels) | **GET** /channels/ | 
**channelsGetTickets**](channels_api.md#channelsGetTickets) | **GET** /channels/{channelid}/tickets | 
**channelsOpenChannel**](channels_api.md#channelsOpenChannel) | **POST** /channels/ | 
**channelsRedeemTickets**](channels_api.md#channelsRedeemTickets) | **POST** /channels/{channelid}/tickets/redeem | 


# **channelsAggregateTickets**
> channelsAggregateTickets(ctx, ctx, channelid)


Takes all acknowledged and winning tickets (if any) from the given channel and aggregates them into a single ticket. Requires cooperation of the ticket issuer.

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **channelid** | **String**|  | 

### Return type

 (empty response body)

### Authorization

[passwordScheme](../README.md#passwordScheme), [keyScheme](../README.md#keyScheme)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **channelsCloseChannel**
> models::ChannelsCloseChannel200Response channelsCloseChannel(ctx, ctx, channelid)


Close a opened channel between this node and other node. Once you've initiated channel closure, you have to wait for a specified closure time, it will show you a closure initiation message with cool-off time you need to wait.   Then you will need to send the same command again to finalize closure. This is a cool down period to give the other party in the channel sufficient time to redeem their tickets.

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **channelid** | **String**|  | 

### Return type

[**models::ChannelsCloseChannel200Response**](channelsCloseChannel_200_response.md)

### Authorization

[passwordScheme](../README.md#passwordScheme), [keyScheme](../README.md#keyScheme)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **channelsFundChannel**
> models::ChannelsFundChannel200Response channelsFundChannel(ctx, ctx, channelid, optional)


Funds an existing channel with the given amount. The channel must be in state OPEN

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **channelid** | **String**|  | 
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **channelid** | **String**|  | 
 **channels_fund_channel_request** | [**ChannelsFundChannelRequest**](ChannelsFundChannelRequest.md)|  | 

### Return type

[**models::ChannelsFundChannel200Response**](channelsFundChannel_200_response.md)

### Authorization

[passwordScheme](../README.md#passwordScheme), [keyScheme](../README.md#keyScheme)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **channelsGetChannel**
> models::ChannelTopology channelsGetChannel(ctx, ctx, channelid)


Returns information about the channel.

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **channelid** | [****](.md)|  | 

### Return type

[**models::ChannelTopology**](ChannelTopology.md)

### Authorization

[passwordScheme](../README.md#passwordScheme), [keyScheme](../README.md#keyScheme)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **channelsGetChannels**
> models::ChannelsGetChannels200Response channelsGetChannels(ctx, ctx, optional)


Lists all active channels between this node and other nodes on the Hopr network. By default response will contain all incomming and outgoing channels that are either open, waiting to be opened, or waiting to be closed. If you also want to receive past channels that were closed, you can pass `includingClosed` in the request url query.

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
 **including_closed** | **String**| When includingClosed is passed the response will include closed channels which are ommited by default. | 
 **full_topology** | **String**| Get the full payment channel graph indexed by the node. | 

### Return type

[**models::ChannelsGetChannels200Response**](channelsGetChannels_200_response.md)

### Authorization

[passwordScheme](../README.md#passwordScheme), [keyScheme](../README.md#keyScheme)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **channelsGetTickets**
> Vec<models::Ticket> channelsGetTickets(ctx, ctx, channelid)


Get tickets earned by relaying data packets by your node for the particular channel.

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **channelid** | **String**|  | 

### Return type

[**Vec<models::Ticket>**](Ticket.md)

### Authorization

[passwordScheme](../README.md#passwordScheme), [keyScheme](../README.md#keyScheme)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **channelsOpenChannel**
> models::ChannelsOpenChannel201Response channelsOpenChannel(ctx, ctx, optional)


Opens a payment channel between this node and the counter party provided. This channel can be used to send messages between two nodes using other nodes on the network to relay the messages. Each message will deduce its cost from the funded amount to pay other nodes for relaying your messages. Opening a channel can take a little bit of time, because it requires some block confirmations on the blockchain.

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
 **channels_open_channel_request** | [**ChannelsOpenChannelRequest**](ChannelsOpenChannelRequest.md)|  | 

### Return type

[**models::ChannelsOpenChannel201Response**](channelsOpenChannel_201_response.md)

### Authorization

[passwordScheme](../README.md#passwordScheme), [keyScheme](../README.md#keyScheme)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **channelsRedeemTickets**
> channelsRedeemTickets(ctx, ctx, channelid)


Redeems your tickets for this channel. Redeeming will change your tickets into Hopr tokens if they are winning ones. You can check how much tickets given channel has by calling /channels/{channelid}/tickets endpoint. Do this before channel is closed as neglected tickets are no longer valid for redeeming.

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **channelid** | **String**|  | 

### Return type

 (empty response body)

### Authorization

[passwordScheme](../README.md#passwordScheme), [keyScheme](../README.md#keyScheme)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

