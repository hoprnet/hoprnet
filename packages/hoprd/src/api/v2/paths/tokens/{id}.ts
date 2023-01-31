import type { Operation } from 'express-openapi'
import { STATUS_CODES } from '../../utils.js'
import { deleteToken } from '../../../token.js'

const DELETE: Operation = [
  async (req, res, _next) => {
    const { node } = req.context
    const { id } = req.params

    try {
      await deleteToken(node.db, id)
      return res.status(204).send()
    } catch (err) {
      return res
        .status(422)
        .send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: err instanceof Error ? err.message : 'Unknown error' })
    }
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
    '422': {
      $ref: '#/components/responses/UnknownFailure'
    }
  }
}

export default { DELETE }
