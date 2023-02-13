import type { Operation } from 'express-openapi'
import { peerIdFromString } from '@libp2p/peer-id'
import { PublicKey } from '@hoprnet/hopr-utils'
import { STATUS_CODES } from '../../utils.js'
import { encodeMessage } from '../../../utils.js'

const POST: Operation = [
  async (req, res, _next) => {
    const message = encodeMessage(req.body.body)
    const recipient = peerIdFromString(req.body.recipient)
    const hops = req.body.hops

    // only set path if given, otherwise a path will be chosen by hopr core
    let path: PublicKey[]
    if (req.body.path != undefined) {
      path = req.body.path.map((peer: string) => PublicKey.fromPeerId(peerIdFromString(peer)))
    }

    try {
      let ackChallenge = await req.context.node.sendMessage(message, recipient, path, hops)
      return res.status(202).json(ackChallenge)
    } catch (err) {
      return res
        .status(422)
        .json({ status: STATUS_CODES.UNKNOWN_FAILURE, error: err instanceof Error ? err.message : 'Unknown error' })
    }
  }
]

POST.apiDoc = {
  description:
    'Send a message to another peer using a given path (list of node addresses that should relay our message through network). If no path is given, HOPR will attempt to find a path.',
  tags: ['Messages'],
  operationId: 'messagesSendMessage',
  requestBody: {
    content: {
      'application/json': {
        schema: {
          type: 'object',
          required: ['body', 'recipient'],
          properties: {
            body: {
              description: 'The message body which should be sent.',
              type: 'string',
              example: 'Hello'
            },
            recipient: {
              description: 'The recipient HOPR peer id, to which the message is sent.',
              type: 'string',
              format: 'peerId',
              example: '16Uiu2HAm2SF8EdwwUaaSoYTiZSddnG4hLVF7dizh32QFTNWMic2b'
            },
            path: {
              description:
                'The path is ordered list of peer ids through which the message should be sent. If no path is provided, a path which covers the nodes minimum required hops will be determined automatically.',
              type: 'array',
              items: {
                description: 'A valid HOPR peer id',
                type: 'string',
                format: 'peerId',
                minItems: 1,
                maxItems: 3,
                example: '16Uiu2HAm1uV82HyD1iJ5DmwJr4LftmJUeMfj8zFypBRACmrJc16n'
              }
            },
            hops: {
              description: 'Number of required intermediate nodes. This parameter is ignored if path is set.',
              type: 'integer',
              minimum: 1,
              maximum: 3,
              example: 3
            }
          }
        }
      }
    }
  },
  responses: {
    '202': {
      description: 'The message was sent successfully. NOTE: This does not imply successful delivery.',
      content: {
        'application/json': {
          schema: {
            type: 'string',
            description: 'Challenge token used to poll for the acknowledgment of the sent message by the first hop.',
            example: 'e61bbdda74873540c7244fe69c39f54e5270bd46709c1dcb74c8e3afce7b9e616d'
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

export default { POST }
