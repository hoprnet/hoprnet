import Hopr from '@hoprnet/hopr-core'
import { ChannelStatus } from '@hoprnet/hopr-utils'
import { Operation } from 'express-openapi'
import PeerId from 'peer-id'

export const closeChannel = async ({ peerId, node }: { peerId: string; node: Hopr }) => {
  let validPeerId: PeerId
  try {
    validPeerId = PeerId.createFromB58String(peerId)
  } catch (err) {
    throw Error('invalidPeerId')
  }

  try {
    const { status, receipt } = await node.closeChannel(validPeerId)
    const smartContractInfo = node.smartContractInfo()
    const channelClosureMins = Math.ceil(smartContractInfo.channelClosureSecs / 60) // convert to minutes

    return {
      channelStatus: status,
      receipt,
      closureWaitTime: status !== ChannelStatus.PendingToClose ? channelClosureMins : undefined
    }
  } catch (err) {
    throw Error('failure' + err.message)
  }
}

export const POST: Operation = [
  async (req, res, _next) => {
    const { node } = req.context
    const { peerId } = req.body

    if (!peerId) {
      return res.status(400).send({ status: 'missingPeerId' })
    }

    try {
      const closureStatus = await closeChannel({ peerId, node })
      return res.status(200).send({ status: 'success', closureStatus })
    } catch (err) {
      return res.status(err.message === 'invalidPeerId' ? 400 : 500).send({ status: err.message })
    }
  }
]

POST.apiDoc = {
  description: 'Close an open channel',
  tags: ['channel'],
  operationId: 'closeChannel',
  requestBody: {
    content: {
      'application/json': {
        schema: {
          type: 'object',
          properties: {
            peerId: { type: 'string', description: 'PeerId attached to the channel that we want to close.' }
          },
          example: {
            peerId: '0x2C505741584f8591e261e59160D0AED5F74Dc29b'
          }
        }
      }
    }
  },
  responses: {
    '200': {
      description: 'Channel closed succesfully',
      content: {
        'application/json': {
          schema: {
            type: 'object',
            properties: {
              status: { type: 'string', example: 'success' },
              closureStatus: {
                $ref: '#/components/schemas/ChannelClosureStatus'
              }
            }
          }
        }
      }
    },
    '400': {
      description: 'Invalid peerId',
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/StatusResponse'
          },
          example: {
            status: 'invalidPeerId | missingPeerId'
          }
        }
      }
    }
  }
}
