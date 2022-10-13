import type { Operation } from 'express-openapi'
import type { Multiaddr } from '@multiformats/multiaddr'
import type Hopr from '@hoprnet/hopr-core'
import { peerIdFromString } from '@libp2p/peer-id'
import { STATUS_CODES } from '../../utils.js'

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
 * Convert `info` taken from `core` to our preferred response format.
 * @param info `entry` information taken from `core`
 * @param multiaddr
 * @returns PeerInfo
 */
function toPeerInfoFormat(info: ReturnType<Hopr['getConnectionInfo']>, multiaddr?: Multiaddr): PeerInfo {
  return {
    peerId: info.id.toString(),
    multiAddr: multiaddr ? multiaddr.toString() : undefined,
    heartbeats: {
      sent: info.heartbeatsSent,
      success: info.heartbeatsSuccess
    },
    lastSeen: info.lastSeen,
    quality: info.quality,
    backoff: info.backoff,
    isNew: info.heartbeatsSent === 0
  }
}

/**
 * @param node a hopr instance
 * @param quality a float range from 0 to 1
 * @returns List of peers alongside their connection status.
 */
export async function getPeers(
  node: Hopr,
  quality: number
): Promise<{
  announced: PeerInfo[]
  connected: PeerInfo[]
}> {
  if (isNaN(quality) || quality < 0 || quality > 1) {
    throw new Error(STATUS_CODES.INVALID_QUALITY)
  }

  try {
    const announced = await node.getAddressesAnnouncedOnChain().then((addrs) => {
      return addrs.reduce((result: Map<string, PeerInfo>, addr: Multiaddr) => {
        const peerId = peerIdFromString(addr.getPeerId())
        try {
          const info = node.getConnectionInfo(peerId)
          // exclude if quality is lesser than the one wanted
          if (info.quality < quality) return result
          result.set(peerId.toString(), toPeerInfoFormat(info, addr))
        } catch {}
        return result
      }, new Map<string, PeerInfo>())
    })

    const connected = [
      ...(function* () {
        for (const peerId of node.getConnectedPeers()) {
          const peerIdStr = peerId.toString()

          // already exists in announced, we use this because it contains multiaddr already
          if (announced.has(peerIdStr)) {
            yield announced.get(peerIdStr)
          } else {
            try {
              const info = node.getConnectionInfo(peerId)
              // exclude if quality is less than the one wanted
              if (info.quality < quality) {
                continue
              }
              yield toPeerInfoFormat(info)
            } catch {}
          }
        }
      })()
    ]

    return {
      connected,
      announced: Array.from(announced.values())
    }
  } catch (err) {
    // Makes sure this doesn't throw
    const errString = `${STATUS_CODES.UNKNOWN_FAILURE} ${err instanceof Error ? err.message : 'Unknown error'}`
    throw new Error(errString)
  }
}

const GET: Operation = [
  async (req, res, _next) => {
    const { node } = req.context
    const quality = parseFloat(String(req.query.quality ?? 0))

    try {
      const info = await getPeers(node, quality)
      return res.status(200).send(info)
    } catch (err) {
      const errString = err instanceof Error ? err.message : err?.toString?.() ?? 'Unknown error'

      if (errString.includes(STATUS_CODES.INVALID_QUALITY)) {
        return res
          .status(400)
          .send({ status: STATUS_CODES.INVALID_QUALITY, error: 'Quality must be a range from 0 to 1' })
      } else {
        return res.status(422).send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: errString })
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

export default { GET }
