# ChannelsGetChannels200Response

## Properties

| Name         | Type                                                   | Description                                                                                       | Notes                        |
| ------------ | ------------------------------------------------------ | ------------------------------------------------------------------------------------------------- | ---------------------------- |
| **incoming** | [**Vec<models::Channel>**](Channel.md)                 | Incomming channels are the ones that were opened by a different node and this node acts as relay. | [optional] [default to None] |
| **outgoing** | [**Vec<models::Channel>**](Channel.md)                 | Outgoing channels are the ones that were opened by this node and is using other node as relay.    | [optional] [default to None] |
| **all**      | [**Vec<models::ChannelTopology>**](ChannelTopology.md) | All the channels indexed by the node in the current network.                                      | [optional] [default to None] |

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
