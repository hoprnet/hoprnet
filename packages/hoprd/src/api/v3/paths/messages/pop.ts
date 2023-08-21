import type { Operation } from 'express-openapi'
import { STATUS_CODES } from '../../utils.js'

const POST: Operation = [
  async (req, res, _next) => {
    const tag: number = req.body.tag
    const msg = await req.context.inbox.pop(tag)

    if (msg) {
      return res.status(200).send({ tag: msg.application_tag, body: new TextDecoder().decode(msg.plain_text) })
    }
    return res.status(404).send()
  }
]

POST.apiDoc = {
  description:
    'Get oldest message currently present in the nodes message inbox. The message is removed from the inbox.',
  tags: ['Messages'],
  operationId: 'messagesPopMessage',
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
      description: 'Returns a message.',
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/ReceivedMessage'
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
    '404': {
      $ref: '#/components/responses/NotFound'
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
