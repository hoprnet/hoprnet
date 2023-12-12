# tickets_api

All URIs are relative to _/api/v3_

| Method                                                         | HTTP request                | Description |
| -------------------------------------------------------------- | --------------------------- | ----------- |
| **ticketsGetStatistics**](tickets_api.md#ticketsGetStatistics) | **GET** /tickets/statistics |
| **ticketsGetTickets**](tickets_api.md#ticketsGetTickets)       | **GET** /tickets/           |
| **ticketsRedeemTickets**](tickets_api.md#ticketsRedeemTickets) | **POST** /tickets/redeem    |

# **ticketsGetStatistics**

> models::TicketsGetStatistics200Response ticketsGetStatistics(ctx, ctx, )

Get statistics regarding all your tickets. Node gets a ticket everytime it relays data packet in channel.

### Required Parameters

This endpoint does not need any parameter.

### Return type

[**models::TicketsGetStatistics200Response**](ticketsGetStatistics_200_response.md)

### Authorization

[passwordScheme](../README.md#passwordScheme), [keyScheme](../README.md#keyScheme)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **ticketsGetTickets**

> Vec<models::Ticket> ticketsGetTickets(ctx, ctx, )

Get all tickets earned by relaying data packets by your node from every channel.

### Required Parameters

This endpoint does not need any parameter.

### Return type

[**Vec<models::Ticket>**](Ticket.md)

### Authorization

[passwordScheme](../README.md#passwordScheme), [keyScheme](../README.md#keyScheme)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **ticketsRedeemTickets**

> ticketsRedeemTickets(ctx, ctx, )

Redeems all tickets from all the channels and exchanges them for Hopr tokens. Every ticket have a chance to be winning one, rewarding you with Hopr tokens.

### Required Parameters

This endpoint does not need any parameter.

### Return type

(empty response body)

### Authorization

[passwordScheme](../README.md#passwordScheme), [keyScheme](../README.md#keyScheme)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
