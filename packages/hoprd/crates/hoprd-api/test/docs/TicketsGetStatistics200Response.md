# TicketsGetStatistics200Response

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**unredeemed** | **f64** | Number of tickets that wait to be redeemed as for Hopr tokens. | [optional] [default to None]
**unredeemed_value** | **String** | Total value of all unredeemed tickets in Hopr tokens. | [optional] [default to None]
**redeemed** | **f64** | Number of tickets already redeemed on this node. | [optional] [default to None]
**redeemed_value** | **String** | Total value of all redeemed tickets in Hopr tokens. | [optional] [default to None]
**losing_tickets** | **f64** | Number of tickets that didn't win any Hopr tokens. To better understand how tickets work read about probabilistic payments (https://docs.hoprnet.org/core/probabilistic-payments) | [optional] [default to None]
**win_proportion** | **f64** | Proportion of number of winning tickets vs loosing tickets, 1 means 100% of tickets won and 0 means that all tickets were losing ones. | [optional] [default to None]
**neglected** | **f64** | Number of tickets that were not redeemed in time before channel was closed. Those cannot be redeemed anymore. | [optional] [default to None]
**neglected_value** | **String** | Total value of all neglected tickets in Hopr tokens. | [optional] [default to None]
**rejected** | **f64** | Number of tickets that were rejected by the network by not passing validation. In other words tickets that look suspicious and are not eligible for redeeming. | [optional] [default to None]
**rejected_value** | **String** | Total value of rejected tickets in Hopr tokens | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


