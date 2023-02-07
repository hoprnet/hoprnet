import type { Operation } from 'express-openapi'
import { authenticateToken, deleteToken } from '../../../token.js'

const DELETE: Operation = [
  async (req, res, _next) => {
    const { node } = req.context
    const { id } = req.params

    const token = await authenticateToken(node.db, id)
    if (token) {
      await deleteToken(node.db, id)
      return res.status(204).send()
    }
    return res.status(404).send()
  }
]

DELETE.apiDoc = {
  description:
    'Deletes a token. Can only be done before the lifetime expired. After the lifetime expired the token is automatically deleted.',
  tags: ['Tokens'],
  operationId: 'tokensDelete',
  parameters: [
    {
      name: 'id',
      in: 'path',
      description: 'ID of the token which shall be deleted.',
      required: true,
      schema: {
        type: 'string',
        example: 'someTOKENid1234'
      }
    }
  ],
  responses: {
    '204': {
      description: 'Token successfully deleted.'
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

export default { DELETE }
