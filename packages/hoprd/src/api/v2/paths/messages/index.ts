import type { Operation } from 'express-openapi'
import PeerId from 'peer-id'
import { PublicKey } from '@hoprnet/hopr-utils'
import { encodeMessage } from '../../../../commands/utils/index.js'
import { STATUS_CODES } from '../../utils.js'

export const POST: Operation = [
  async (req, res, _next) => {
    const message = encodeMessage(req.body.body)
    const recipient: PeerId = PeerId.createFromB58String(req.body.recipient)

    // only set path if given, otherwise a path will be chosen by hopr core
    let path: PublicKey[]
    if (req.body.path != undefined) {
      path = req.body.path.map((peer) => PublicKey.fromPeerId(PeerId.createFromB58String(peer)))
    }

    try {
      await req.context.node.sendMessage(message, recipient, path)
      res.status(204).send()
    } catch (err) {
      res.status(422).json({ status: STATUS_CODES.UNKNOWN_FAILURE, error: err.message })
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
            }
          }
        }
      }
    }
  },
  responses: {
    '204': {
      description: 'The message was sent successfully. NOTE: This does not imply successful delivery.'
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
