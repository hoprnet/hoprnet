import type { Operation } from 'express-openapi'
import { STATUS_CODES } from '../../utils.js'

const POST: Operation = [
  async (req, res, _next) => {
    const tag = req.body.tag
    const msgs = await req.context.inbox.pop_all(tag)
    const messages = msgs.map((m) => {
      return { tag: m.application_tag, body: new TextDecoder().decode(m.plain_text) }
    })

    return res.status(200).send({ messages })
  }
]

POST.apiDoc = {
  description:
    'Get list of messages currently present in the nodes message inbox. The messages are removed from the inbox.',
  tags: ['Messages'],
  operationId: 'messagesPopAllMessage',
  requestBody: {
    content: {
      'application/json': {
        schema: {
          type: 'object',
          required: ['tag'],
          properties: {
            tag: {
              $ref: '#/components/schemas/MessageTag'
            }
          }
        }
      }
    }
  },
  responses: {
    '200': {
      description: 'Returns list of messages.',
      content: {
        'application/json': {
          schema: {
            type: 'object',
            required: ['messages'],
            properties: {
              messages: {
                type: 'array',
                items: {
                  $ref: '#/components/schemas/ReceivedMessage'
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

export default { POST }
