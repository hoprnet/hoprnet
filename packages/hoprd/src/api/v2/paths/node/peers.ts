import type { Operation } from 'express-openapi'
import Hopr from '@hoprnet/hopr-core'
import PeerId from 'peer-id'
import { STATUS_CODES } from '../../utils'

export type PeerInfo = {
  peerId: string
  multiAddr?: string
  heartbeats: {
    sent: number
    success: number
  }
  lastSeen: number
  quality: number
  backoff: number
  isNew: boolean
}

/**
 * @param node a hopr instance
 * @param quality a float range from 0 to 1
 * @returns List of peers alongside their connection status.
 */
export const getPeers = async (
  node: Hopr,
  quality: number
): Promise<{
  announced: PeerInfo[]
  connected: PeerInfo[]
}> => {
  if (isNaN(quality) || quality > 1) throw new Error(STATUS_CODES.INVALID_QUALITY)

  try {
    const connected = node.getConnectedPeers().reduce<PeerInfo[]>((result, peerId) => {
      try {
        const info = node.getConnectionInfo(peerId)
        // exclude if quality is lesser than the one wanted
        if (info.quality < quality) return result
        const isNew = info.heartbeatsSent === 0

        result.push({
          peerId: info.id.toB58String(),
          heartbeats: {
            sent: info.heartbeatsSent,
            success: info.heartbeatsSuccess
          },
          lastSeen: info.lastSeen,
          quality: info.quality,
          backoff: info.backoff,
          isNew
        })
      } catch {}
      return result
    }, [])

    const announced = await node.getAddressesAnnouncedOnChain().then((addrs) => {
      return addrs.reduce<PeerInfo[]>((result, addr) => {
        const peerId = PeerId.createFromB58String(addr.getPeerId())
        try {
          const info = node.getConnectionInfo(peerId)
          // exclude if quality is lesser than the one wanted
          if (info.quality < quality) return result
          const isNew = info.heartbeatsSent === 0

          result.push({
            peerId: info.id.toB58String(),
            multiAddr: addr.toString(),
            heartbeats: {
              sent: info.heartbeatsSent,
              success: info.heartbeatsSuccess
            },
            lastSeen: info.lastSeen,
            quality: info.quality,
            backoff: info.backoff,
            isNew
          })
        } catch {}
        return result
      }, [])
    })

    return {
      connected,
      announced
    }
  } catch (error) {
    throw new Error(STATUS_CODES.UNKNOWN_FAILURE + ' ' + error.message)
  }
}

export const GET: Operation = [
  async (req, res, _next) => {
    const { node } = req.context
    const quality = parseFloat(String(req.query.quality ?? 0))

    try {
      const info = await getPeers(node, quality)
      return res.status(200).send(info)
    } catch (error) {
      if (error.message.includes(STATUS_CODES.INVALID_QUALITY)) {
        return res
          .status(400)
          .send({ status: STATUS_CODES.INVALID_QUALITY, error: 'Quality must be a range from 0 to 1' })
      } else {
        return res.status(422).send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: error.message })
      }
    }
  }
]

const PEER_INFO_DOC: any = {
  type: 'object',
  properties: {
    peerId: {
      $ref: '#/components/schemas/HoprAddress'
    },
    multiAddr: {
      $ref: '#/components/schemas/MultiAddress'
    },
    heartbeats: {
      type: 'object',
      properties: {
        sent: {
          type: 'number',
          description: 'Heartbeats sent to the node',
          example: 10
        },
        success: {
          type: 'number',
          description: 'Successful heartbeats sent to the node',
          example: 8
        }
      }
    },
    lastSeen: {
      type: 'number',
      description: 'Timestamp on when the node was last seen (in milliseconds)',
      example: 1646410980793
    },
    quality: {
      type: 'number',
      description:
        "A float between 0 (completely unreliable) and 1 (completely reliable) estimating the quality of service of a peer's network connection",
      example: 0.8
    },
    backoff: {
      type: 'number'
    },
    isNew: {
      type: 'boolean',
      description: 'True if the node is new (no heartbeats sent yet).'
    }
  }
}

GET.apiDoc = {
  description:
    'Lists information for `connected peers` and `announced peers`.\nConnected peers are nodes which are connected to the node while announced peers are nodes which have announced to the network.\nOptionally, you can pass `quality` parameter which would filter out peers with lower quality to the one specified.',
  tags: ['Node'],
  operationId: 'nodeGetPeers',
  parameters: [
    {
      in: 'query',
      name: 'quality',
      description:
        'When quality is passed, the response will only include peers with higher or equal quality to the one specified.',
      schema: {
        type: 'number',
        example: '0.5'
      }
    }
  ],
  responses: {
    '200': {
      description: 'Peers information fetched successfuly.',
      content: {
        'application/json': {
          schema: {
            type: 'object',
            properties: {
              connected: {
                type: 'array',
                items: PEER_INFO_DOC
              },
              announced: {
                type: 'array',
                items: PEER_INFO_DOC
              }
            }
          }
        }
      }
    },
    '400': {
      description: `Invalid input. One of the parameters passed is in an incorrect format.`,
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/RequestStatus'
          },
          example: {
            status: STATUS_CODES.INVALID_QUALITY
          }
        }
      }
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
