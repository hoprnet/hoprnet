import type { Operation } from 'express-openapi'

const GET: Operation = [
  async (req, res, _next) => {
    const { token } = req.context
    if (token) {
      return res.status(200).send(token)
    }
    return res.status(404).send()
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
    '403': {
      $ref: '#/components/responses/Forbidden'
    },
    '404': {
      $ref: '#/components/responses/NotFound'
    }
  }
}

export default { GET }
