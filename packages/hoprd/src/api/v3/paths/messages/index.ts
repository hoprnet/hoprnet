import type { Operation } from 'express-openapi'
import { peerIdFromString } from '@libp2p/peer-id'
import { create_counter, OffchainPublicKey, debug } from '@hoprnet/hopr-utils'

import { STATUS_CODES } from '../../utils.js'
import { encodeMessage } from '../../../utils.js'
import { RPCH_MESSAGE_REGEXP } from '../../../v3.js'

const log = debug('hoprd:api:v3:messages')

const metric_successfulSendApiCalls = create_counter(
  'hoprd_counter_api_successful_send_msg',
  'Number of successful API calls to POST message endpoint'
)
const metric_failedSendApiCalls = create_counter(
  'hoprd_counter_api_failed_send_msg',
  'Number of failed API calls to POST message endpoint'
)

const DELETE: Operation = [
  async (req, res, _next) => {
    const tag: number = req.query.tag as unknown as number

    // the popped messages are ignored
    // @ts-ignore unused-variable
    const _messages = await req.context.inbox.pop_all(tag)

    return res.status(204).send()
  }
]

const POST: Operation = [
  async (req, res, _next) => {
    const message = encodeMessage(req.body.body)
    const recipient = peerIdFromString(req.body.peerAddress)
    const hops = req.body.hops

    const tag = req.body.tag

    // only set path if given, otherwise a path will be chosen by hopr core
    let path: OffchainPublicKey[]
    if (req.body.path != undefined) {
      path = req.body.path.map((peer: string) => OffchainPublicKey.from_peerid_str(peer))
    }

    try {
      let ackChallenge = await req.context.node.sendMessage(message, recipient, path, hops, tag)
      log(`after sending message`)
      metric_successfulSendApiCalls.increment()
      return res.status(202).json(ackChallenge)
    } catch (err) {
      log('error while sending message', err)
      metric_failedSendApiCalls.increment()
      let msg = req.body.body
      if (RPCH_MESSAGE_REGEXP.test(msg)) {
        log(`RPCh: failed to send message [${msg}]`)
      }
      return res
        .status(422)
        .json({ status: STATUS_CODES.UNKNOWN_FAILURE, error: err instanceof Error ? err.message : 'Unknown error' })
    }
  }
]

DELETE.apiDoc = {
  description: 'Delete messages from nodes message inbox. Does not return any data.',
  tags: ['Messages'],
  operationId: 'messagesDeleteMessages',
  parameters: [
    {
      in: 'query',
      name: 'tag',
      description: 'Tag used to filter target messages.',
      required: true,
      schema: {
        $ref: '#/components/schemas/MessageTag'
      }
    }
  ],
  responses: {
    '204': {
      description: 'Messages successfully deleted.'
    },
    '401': {
      $ref: '#/components/responses/Unauthorized'
    },
    '403': {
      $ref: '#/components/responses/Forbidden'
    }
  }
}

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
          required: ['tag', 'body', 'peerAddress'],
          properties: {
            tag: {
              $ref: '#/components/schemas/MessageTag'
            },
            body: {
              $ref: '#/components/schemas/MessageBody'
            },
            peerAddress: {
              description: 'The recipient HOPR peer id, to which the message is sent.',
              type: 'string',
              format: 'peerid',
              example: '12Diu2HAm2SF8EdwwUaaSoYTiZSddnG4hLVF'
            },
            path: {
              description:
                'The path is ordered list of peer ids through which the message should be sent. If no path is provided, a path which covers the nodes minimum required hops will be determined automatically.',
              type: 'array',
              items: {
                description: 'A valid HOPR peer id',
                type: 'string',
                format: 'peerid',
                minItems: 0,
                maxItems: 3,
                example: '12Diu2HAm1uV82HyD1iJ5DmwJr4LftmJUeMf'
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

export default { DELETE, POST }
