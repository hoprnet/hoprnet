import type { Operation } from 'express-openapi'

import { type ChannelEntry, channelStatusToString } from '@hoprnet/hopr-utils'
import { STATUS_CODES } from '../../utils.js'

export interface ChannelTopologyInfo {
  channelId: string
  sourcePeerId: string
  destinationPeerId: string
  sourceAddress: string
  destinationAddress: string
  balance: string
  status: string
  commitment: string
  ticketEpoch: string
  ticketIndex: string
  channelEpoch: string
  closureTime: string
}

/**
 * Format channel entries
 * @param channel channelEntry entity saved in the database
 * @returns stringified fields from ChannelEntry and both peer id and address for source/destination
 */
export const formatChannelTopologyInfo = (channel: ChannelEntry): ChannelTopologyInfo => {
  return {
    channelId: channel.getId().toHex(),
    sourcePeerId: channel.source.toPeerId().toString(),
    destinationPeerId: channel.destination.toPeerId().toString(),
    sourceAddress: channel.source.toAddress().toHex(),
    destinationAddress: channel.destination.toAddress().toHex(),
    balance: channel.balance.toBN().toString(),
    status: channelStatusToString(channel.status),
    commitment: channel.commitment.toHex(),
    ticketEpoch: channel.ticketEpoch.toBN().toString(),
    ticketIndex: channel.ticketIndex.toBN().toString(),
    channelEpoch: channel.channelEpoch.toBN().toString(),
    closureTime: channel.closureTime.toBN().toString()
  }
}

/**
 * @returns the current on-chain channel topology, i.e. list of channels seen by the node
 */
const GET: Operation = [
  async (req, res, _next) => {
    const { node } = req.context

    try {
      const channels = await node.getAllChannels()
      // format channels
      const channelTopology: ChannelTopologyInfo[] = channels.map(formatChannelTopologyInfo)
      return res.status(200).send(channelTopology)
    } catch (err) {
      return res
        .status(422)
        .send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: err instanceof Error ? err.message : 'Unknown error' })
    }
  }
]

GET.apiDoc = {
  description: 'Get the full payment channel graph indexed by the node.',
  tags: ['Topology'],
  operationId: 'topologyGetChannels',
  responses: {
    '200': {
      description: 'Topology fetched successfully',
      content: {
        'application/json': {
          schema: {
            type: 'array',
            items: { $ref: '#/components/schemas/TopologyChannel' },
            description: 'All the channels indexed by the node in the current network.'
          }
        }
      }
    },
    '401': {
      $ref: '#/components/responses/Unauthorized'
    },
    '403': {
      $ref: '#/components/responses/Forbidden'
    },
    '404': {
      $ref: '#/components/responses/NotFound'
    }
  }
}

export default { GET }
