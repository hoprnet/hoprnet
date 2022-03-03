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
  success?: string
}

/**
 * @param info
 * @returns a measurement of success (in %), returns undefined if node is newly added
 */
export const measureSuccess = (info: { heartbeatsSent: number; heartbeatsSuccess: number }): string => {
  return info.heartbeatsSent > 0 ? ((info.heartbeatsSuccess / info.heartbeatsSent) * 100).toFixed() + '%' : undefined
}

/**
 * @returns List of peers alongside their connection status.
 */
export const getPeers = async (
  node: Hopr
): Promise<{
  announced: PeerInfo[]
  connected: PeerInfo[]
}> => {
  try {
    const connected = node.getConnectedPeers().reduce<PeerInfo[]>((result, peerId) => {
      try {
        const info = node.getConnectionInfo(peerId)
        const isNew = info.heartbeatsSent === 0
        result.push({
          peerId: info.id.toB58String(),
          heartbeats: {
            sent: info.heartbeatsSent,
            success: info.heartbeatsSuccess
          },
          lastSeen: info.lastSeen,
          quality: info.lastTen,
          backoff: info.backoff,
          isNew,
          success: isNew ? measureSuccess(info) : undefined
        })
      } catch {}
      return result
    }, [])

    const announced = await node.getAnnouncedAddresses().then((addrs) => {
      return addrs.reduce<PeerInfo[]>((result, addr) => {
        const peerId = PeerId.createFromB58String(addr.getPeerId())
        try {
          const info = node.getConnectionInfo(peerId)
          const isNew = info.heartbeatsSent === 0
          result.push({
            peerId: info.id.toB58String(),
            multiAddr: addr.toString(),
            heartbeats: {
              sent: info.heartbeatsSent,
              success: info.heartbeatsSuccess
            },
            lastSeen: info.lastSeen,
            quality: info.lastTen,
            backoff: info.backoff,
            isNew,
            success: isNew ? measureSuccess(info) : undefined
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
    throw new Error(STATUS_CODES.UNKNOWN_FAILURE + error.message)
  }
}

export const GET: Operation = [
  async (req, res, _next) => {
    const { node } = req.context

    try {
      const info = await getPeers(node)
      return res.status(200).send(info)
    } catch (error) {
      return res.status(422).send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: error.message })
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
          describe: 'Heartbeats sent to the node'
        },
        success: {
          type: 'number',
          describe: 'Successful heartbeats sent to the node'
        }
      }
    },
    lastSeen: {
      type: 'number',
      describe: 'Timestamp on when the node was last seen (in milliseconds)'
    },
    quality: {
      type: 'number',
      describe:
        "A float between 0 (completely unreliable) and 1 (completely reliable) estimating the quality of service of a peer's network connection"
    },
    backoff: {
      type: 'number'
    },
    isNew: {
      type: 'boolean',
      describe: 'True if the node is new (no heartbeats sent yet).'
    },
    success: {
      type: 'string',
      describe: 'A percentage of how much success there is connecting to the node.'
    }
  }
}

GET.apiDoc = {
  description:
    'Lists information for `connected peers` and `announced peers`.\nConnected peers are nodes which are connected to the node while announced peers are nodes which have announced to the network.',
  tags: ['Node'],
  operationId: 'nodeGetPeers',
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
