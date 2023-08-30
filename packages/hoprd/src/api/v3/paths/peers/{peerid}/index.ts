import type { Hopr } from '@hoprnet/hopr-core'
import type { Operation } from 'express-openapi'
import type { PeerId } from '@libp2p/interface-peer-id'
import { peerIdFromString } from '@libp2p/peer-id'
import { STATUS_CODES } from '../../../utils.js'

export const getPeerInfo = async (node: Hopr, peerId: PeerId) => {
  const announced = await node.getAddressesAnnouncedToDHT(peerId)
  const observed = await node.getObservedAddresses(peerId)

  return {
    announced: announced.map((v) => v.toString()),
    observed: observed.map((v) => v.toString())
  }
}

export const GET: Operation = [
  async (req, res, _next) => {
    const { node }: { node: Hopr } = req.context
    const { peerid } = req.params

    try {
      const info = await getPeerInfo(node, peerIdFromString(peerid))
      return res.status(200).send(info)
    } catch (err) {
      return res
        .status(422)
        .send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: err instanceof Error ? err.message : 'Unknown error' })
    }
  }
]

GET.apiDoc = {
  description: 'Get information about this peer.',
  tags: ['PeerInfo'],
  operationId: 'peerInfoGetPeerInfo',
  parameters: [
    {
      in: 'path',
      name: 'peerid',
      required: true,
      schema: {
        format: 'peerid',
        type: 'string'
      }
    }
  ],
  responses: {
    '200': {
      description: 'Peer information fetched successfully.',
      content: {
        'application/json': {
          schema: {
            type: 'object',
            properties: {
              announced: {
                type: 'array',
                items: {
                  $ref: '#/components/schemas/MultiAddress'
                }
              },
              observed: {
                type: 'array',
                items: {
                  $ref: '#/components/schemas/MultiAddress'
                }
              }
            }
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
    '422': {
      description: 'Unknown failure.',
      content: {
        'application/json': {
          schema: {
            type: 'object',
            properties: {
              status: { type: 'string', example: STATUS_CODES.UNKNOWN_FAILURE },
              error: { type: 'string', example: 'Full error message.' }
            }
          },
          example: { status: STATUS_CODES.UNKNOWN_FAILURE, error: 'Full error message.' }
        }
      }
    }
  }
}
