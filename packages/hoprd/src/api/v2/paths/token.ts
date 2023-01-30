
import type { Operation } from 'express-openapi'
import type { State, StateOps } from '../../../../types.js'
import { STATUS_CODES } from '../../utils.js'

const GET: Operation = [
  async (req, res, _next) => {
    const { stateOps } = req.context
    const { alias } = req.params

    try {
      const peerId = getAlias(stateOps.getState(), alias as string)
      return res.status(200).send({ peerId })
    } catch (err) {
      const errString = err instanceof Error ? err.message : err?.toString?.() ?? 'Unknown error'

      if (errString.includes(STATUS_CODES.PEERID_NOT_FOUND)) {
        return res.status(404).send({ status: STATUS_CODES.PEERID_NOT_FOUND })
      } else {
        return res.status(422).send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: errString })
      }
    }
  }
]

GET.apiDoc = {
  description: 'Get the PeerId (Hopr address) that have this alias assigned to it.',
  tags: ['Aliases'],
  operationId: 'aliasesGetAlias',
  parameters: [
    {
      name: 'alias',
      in: 'path',
      description: 'Alias that we previously assigned to some PeerId.',
      required: true,
      schema: {
        type: 'string',
        example: 'Alice'
      }
    }
  ],
  responses: {
    '200': {
      description: 'Token information.',
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schema/Token'
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
