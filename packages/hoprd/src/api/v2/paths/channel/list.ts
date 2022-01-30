import type { Operation } from 'express-openapi'
import type Hopr from '@hoprnet/hopr-core'
import { ChannelStatus, PublicKey } from '@hoprnet/hopr-utils'
import { STATUS_CODES } from '../../'

export interface ChannelInfo {
  type: 'outgoing' | 'incoming'
  channelId: string
  peerId: string
  status: string
  balance: string
}

const channelStatusToString = (status: ChannelStatus): string => {
  if (status === 0) return 'Closed'
  else if (status === 1) return 'WaitingForCommitment'
  else if (status === 2) return 'Open'
  else if (status === 3) return 'PendingToClose'
  return 'Unknown'
}

export const listChannels = async (node: Hopr) => {
  const selfPubKey = new PublicKey(node.getId().pubKey.marshal())
  const selfAddress = selfPubKey.toAddress()

  const channelsFrom: ChannelInfo[] = (await node.getChannelsFrom(selfAddress))
    .filter((channel) => channel.status !== ChannelStatus.Closed)
    .map((channel) => ({
      type: 'incoming',
      channelId: channel.getId().toHex(),
      peerId: channel.destination.toPeerId().toB58String(),
      status: channelStatusToString(channel.status),
      balance: channel.balance.toBN().toString()
    }))

  const channelsTo: ChannelInfo[] = (await node.getChannelsTo(selfAddress))
    .filter((channel) => channel.status !== ChannelStatus.Closed)
    .map((channel) => ({
      type: 'outgoing',
      channelId: channel.getId().toHex(),
      peerId: channel.source.toPeerId().toB58String(),
      status: channelStatusToString(channel.status),
      balance: channel.balance.toBN().toString()
    }))

  return { incoming: channelsFrom, outgoing: channelsTo }
}

export const GET: Operation = [
  async (req, res, _next) => {
    const { node } = req.context

    try {
      const channels = await listChannels(node)
      return res.status(200).send({ channels })
    } catch (err) {
      return res.status(500).send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: err.message })
    }
  }
]

GET.apiDoc = {
  description: 'Lists your channels.',
  tags: ['channel'],
  operationId: 'channelList',
  responses: {
    '200': {
      description: 'Channels fetched succesfully.',
      content: {
        'application/json': {
          schema: {
            type: 'object',
            properties: {
              channels: {
                type: 'object',
                properties: {
                  incoming: { type: 'array', items: { $ref: '#/components/schemas/Channel' } },
                  outgoing: { type: 'array', items: { $ref: '#/components/schemas/Channel' } }
                }
              }
            }
          }
        }
      }
    }
  }
}
