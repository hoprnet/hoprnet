import type { Operation } from 'express-openapi'

const GET: Operation = [
  async (req, res, _next) => {
    const { token } = req.context
    return res.status(200).send(token)
  }
]

GET.apiDoc = {
  description: 'Get the full token information for the token used in authentication.',
  tags: ['Tokens'],
  operationId: 'tokensGetToken',
  responses: {
    '200': {
      description: 'Token information.',
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/Token'
          }
        }
      }
    },
        '401': {
          $ref: '#/components/responses/Unauthorized'
        },
        '422': {
          $ref: '#/components/responses/UnknownFailure'
        }
      }
    }

export default { GET }
