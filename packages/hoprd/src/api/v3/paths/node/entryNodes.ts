import type { Operation } from 'express-openapi'
import type { Multiaddr } from '@multiformats/multiaddr'
import type { Hopr } from '@hoprnet/hopr-core'
import { STATUS_CODES } from '../../utils.js'
import type { PeerId } from '@libp2p/interface-peer-id'

export type EntryNodeInfo = {
  [index: string]: {
    multiaddrs: string[]
    isEligible: boolean
  }
}

export async function getEntryNodes(node: Hopr): Promise<EntryNodeInfo> {
  const entryNodes = (await node.getEntryNodes()) as { id: PeerId; multiaddrs: Multiaddr[]; isEligible: boolean }[]

  const result: EntryNodeInfo = {}
  for (const entryNode of entryNodes) {
    result[entryNode.id.toString()] = {
      multiaddrs: entryNode.multiaddrs.map((ma: Multiaddr) => ma.toString()),
      isEligible: await node.isAllowedAccessToNetwork(entryNode.id)
    }
  }

  return result
}

const GET: Operation = [
  async (req, res, _next) => {
    const { node }: { node: Hopr } = req.context

    try {
      const info = await getEntryNodes(node)
      return res.status(200).send(info)
    } catch (err) {
      const errString = err instanceof Error ? err.message : err?.toString?.() ?? 'Unknown error'

      return res.status(422).send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: errString })
    }
  }
]

const PEER_INFO_DOC: any = {
  type: 'object',
  properties: {
    multiaddrs: {
      type: 'array',
      items: {
        type: 'string'
      },
      description: 'Known Multiaddrs of the node'
    },
    isEligible: {
      type: 'boolean',
      description: 'true if peer is allowed to access network, otherwise false'
    }
  }
}

GET.apiDoc = {
  description: 'List all known entry nodes and their multiaddrs and their eligibility state',
  tags: ['Node'],
  operationId: 'nodeGetEntryNodes',
  responses: {
    '200': {
      description: 'Entry node information fetched successfuly.',
      content: {
        'application/json': {
          schema: {
            type: 'object',
            additionalProperties: PEER_INFO_DOC
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
