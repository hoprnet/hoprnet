import { peerIdFromString } from '@libp2p/peer-id'
import { PEER_METADATA_PROTOCOL_VERSION, PeerStatus, type Hopr } from '@hoprnet/hopr-core'
import { debug, AccountEntry, Address } from '@hoprnet/hopr-utils'
import { STATUS_CODES } from '../../utils.js'
import { Multiaddr } from '@multiformats/multiaddr'

import type { Operation } from 'express-openapi'

const log = debug('hoprd:api:v3:node-peers')

export type PeerInfo = {
  peerId: string
  peerAddress: string
  multiAddr?: string
  heartbeats: {
    sent: number
    success: number
  }
  lastSeen: number
  quality: number
  backoff: number
  isNew: boolean
  reportedVersion: string
}

/**
 * Convert `info` taken from `core` to our preferred response format.
 * @param info `entry` information taken from `core`
 * @param multiaddr
 * @returns PeerInfo
 */
export function toPeerInfoFormat(address: Address | undefined, info: PeerStatus, multiaddr?: Multiaddr): PeerInfo {
  return {
    peerId: info.peer_id(),
    peerAddress: address?.to_string(),
    multiAddr: multiaddr ? multiaddr.toString() : '',
    heartbeats: {
      sent: Number(info.heartbeats_sent),
      success: Number(info.heartbeats_succeeded)
    },
    lastSeen: Number(info.last_seen),
    quality: info.quality,
    backoff: info.backoff,
    isNew: info.heartbeats_sent === BigInt(0),
    reportedVersion: info.metadata().get(PEER_METADATA_PROTOCOL_VERSION) ?? 'unknown'
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
    const announcedMap = new Map<string, PeerInfo>()

    let accounts: AsyncGenerator<AccountEntry, void, void> = node.getAccountsAnnouncedOnChain()
    for await (const acc of accounts) {
      const peerId = peerIdFromString(acc.public_key.to_peerid_str())
      const info = await node.getConnectionInfo(peerId)
      // exclude if quality is lesser than the one wanted
      if (info === undefined || info.quality < quality) {
        continue
      }
      announcedMap.set(
        peerId.toString(),
        toPeerInfoFormat(acc.chain_addr, info, new Multiaddr(acc.get_multiaddr_str()))
      )
    }

    let connected_peers = await node.getConnectedPeers()

    const connected = []

    for (const peerId of connected_peers) {
      const peerIdStr = peerId.toString()
      let chainKey: Address
      try {
        chainKey = await node.peerIdToChainKey(peerId)
      } catch {
        // chain key might not be available if key binding is missing
      }
      // already exists in announced, we use this because it contains multiaddr already
      if (announcedMap.has(peerIdStr)) {
        connected.push(announcedMap.get(peerIdStr))
      } else {
        const info = await node.getConnectionInfo(peerId)
        // exclude if quality is less than the one wanted
        if (info === undefined || info.quality < quality) {
          continue
        }
        connected.push(toPeerInfoFormat(chainKey, info))
      }
    }

    return {
      connected,
      announced: [...announcedMap.values()]
    }
  } catch (err) {
    log(`Error while getting all peers address information: ${err}`)
    const errString = `${STATUS_CODES.UNKNOWN_FAILURE} ${err instanceof Error ? err.message : 'Unknown error'}`
    throw new Error(errString)
  }
}

const GET: Operation = [
  async (req, res, _next) => {
    const { node }: { node: Hopr } = req.context
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
    peerAddress: {
      $ref: '#/components/schemas/NativeAddress'
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
    },
    reportedVersion: {
      type: 'string',
      example: '1.92.12',
      description:
        'HOPR protocol version as determined from the successful ping in the Major.Minor.Patch format or "unknown"'
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

export default { GET }
