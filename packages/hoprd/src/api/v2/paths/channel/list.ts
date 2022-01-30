import { Operation } from 'express-openapi'

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

export const listOpenChannels = async ({ node }: { node: Hopr }) => {
  try {
    const selfPubKey = new PublicKey(node.getId().pubKey.marshal())
    const selfAddress = selfPubKey.toAddress()

    const channelsFrom: ChannelInfo[] = (await node.getChannelsFrom(selfAddress))
      .filter((channel) => channel.status !== ChannelStatus.Closed)
      .map(({ /*getId,*/ destination, status, balance }) => ({
        type: 'incoming',
        channelId: 'getId().toHex()', // <<<--- NOTE: issue here, source is undefined
        peerId: destination.toPeerId().toB58String(),
        status: channelStatusToString(status),
        balance: balance.toFormattedString()
      }))

    const channelsTo: ChannelInfo[] = (await node.getChannelsTo(selfAddress))
      .filter((channel) => channel.status !== ChannelStatus.Closed)
      .map(({ /*getId,*/ source, status, balance }) => ({
        type: 'outgoing',
        channelId: 'getId().toHex()', // <<<--- NOTE: issue here, source is undefined
        peerId: source.toPeerId().toB58String(),
        status: channelStatusToString(status),
        balance: balance.toFormattedString()
      }))

    return { incoming: channelsFrom, outgoing: channelsTo }
  } catch (err) {
    return new Error(err.message)
  }
}

export const GET: Operation = [
  async (req, res, _next) => {
    const { node } = req.context

    const channels = await listOpenChannels({ node })
    if (isError(channels)) {
      return res.status(500).send({ status: channels.message })
    } else {
      return res.status(200).send({ status: 'success', channels })
    }
  }
]

GET.apiDoc = {
  description: 'Lists your currently open channels',
  tags: ['channel'],
  operationId: 'listOpenChannels',
  parameters: [],
  responses: {
    '200': {
      description: 'Channels fetched succesfully',
      content: {
        'application/json': {
          schema: {
            type: 'object',
            properties: {
              status: { type: 'string', example: 'success' },
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
    },
    '500': {
      description: 'No alias found for the peerId',
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/StatusResponse'
          },
          example: { status: 'failure' }
        }
      }
    }
  }
}
