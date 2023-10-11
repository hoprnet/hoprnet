import { peerIdFromString } from '@libp2p/peer-id'
import { Hopr, peer_metadata_protocol_version_name } from '@hoprnet/hopr-utils'

import { STATUS_CODES } from '../../../utils.js'

import type { Operation } from 'express-openapi'

/**
 * Pings another peer to check its availability.
 * @returns Latency and HOPR protocol version (once known) if ping was successful.
 */
export const ping = async ({ node, peerId }: { node: Hopr; peerId: string }) => {
  let validPeerId: string
  try {
    validPeerId = peerIdFromString(peerId).toString()
  } catch (err) {
    throw Error(STATUS_CODES.INVALID_PEERID)
  }

  let pingResult: Awaited<ReturnType<Hopr['ping']>>
  let error: any

  try {
    pingResult = await node.ping(validPeerId)
  } catch (err) {
    error = err
  }

  if (error && error.message) {
    throw error
  }

  if (pingResult !== undefined && pingResult >= 0) {
    const info = await node.getPeerInfo(validPeerId)
    let protocol_version = info.metadata().get(peer_metadata_protocol_version_name()) ?? 'unknown'

    return { latency: pingResult, reportedVersion: protocol_version }
  }

  throw Error(STATUS_CODES.TIMEOUT)
}

const POST: Operation = [
  async (req, res, _next) => {
    const { node }: { node: Hopr } = req.context
    const { peerid } = req.params

    try {
      const pingRes = await ping({ peerId: peerid, node })
      return res.status(200).send(pingRes)
    } catch (err) {
      const errString = err instanceof Error ? err.message : 'Unknown error'

      if (STATUS_CODES[errString]) {
        return res.status(422).send({ status: STATUS_CODES[errString] })
      } else {
        return res.status(422).send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: errString })
      }
    }
  }
]

POST.apiDoc = {
  description: 'Pings another node to check its availability.',
  tags: ['Peers'],
  operationId: 'peersPingPeer',
  parameters: [
    {
      name: 'peerid',
      in: 'path',
      description: 'Peer id that should be pinged',
      required: true,
      schema: {
        format: 'peerid',
        type: 'string'
      }
    }
  ],
  responses: {
    '200': {
      description: 'Ping successful.',
      content: {
        'application/json': {
          schema: {
            type: 'object',
            properties: {
              latency: {
                type: 'number',
                example: 10,
                description: 'Number of milliseconds it took to get the response from the pinged node.'
              },
              reportedVersion: {
                type: 'string',
                example: '1.92.12',
                description:
                  'HOPR protocol version as determined from the successful ping in the Major.Minor.Patch format or "unknown"'
              }
            }
          }
        }
      }
    },
    '400': {
      description: 'Invalid peerId.',
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/RequestStatus'
          },
          example: { status: STATUS_CODES.INVALID_PEERID }
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
      description: `An error occured (see error details) or timeout - node with specified PeerId didn't respond in time.`,
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/RequestStatus'
          },
          example: { status: STATUS_CODES.TIMEOUT }
        }
      }
    }
  }
}

export default { POST }
