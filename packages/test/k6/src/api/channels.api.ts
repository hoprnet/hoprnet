import http, { RefinedResponse } from 'k6/http'

import { ChannelResponse, HoprNode } from './hoprd.types'
import { HoprEntityApi } from './hopr-entity-api'

/**
 * Wrap actions on hopr channels
 */
export class ChannelsApi extends HoprEntityApi {
  public constructor(node: HoprNode) {
    super(node)
    this.addRequestHeader('entity', 'channels')
  }

  /**
   * Invoke API Rest call for getting node channels
   * @returns HTTP response in text mode
   */
  public getChannels(): RefinedResponse<'text'> {
    this.addRequestHeader('action', 'channels')

    const response: RefinedResponse<'text'> = http.get(`${this.node.url}/channels`, this.params)

    if (response.status === 200) {
      const channelsResponse: ChannelResponse = JSON.parse(response.body)
      console.log(
        `Hopr node ${this.node.alias} has ${channelsResponse.incoming.length} incomming channels and ${channelsResponse.outgoing.length} outgoing channels`
      )
    }

    this.sleep(this.node.sleepTime.defaultMin, this.node.sleepTime.defaultMax)
    return response
  }
}
