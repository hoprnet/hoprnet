import { Operation } from 'express-openapi'
import { isError } from '../../../../commands/v2'
import { getAlias } from '../../../../commands/v2/logic/alias'

export const parameters = []

export const GET: Operation = [
  async (req, res, _next) => {
    const { commands } = req.context
    const { peerId } = req.query

    if (!peerId) {
      return res.status(400).send({ status: 'noPeerIdProvided' })
    }

    const aliases = getAlias({ peerId: peerId as string, state: commands.state })
    if (isError(aliases)) {
      return res.status(404).send({ status: 'aliasNotFound' })
    } else {
      return res.status(200).send({ status: 'success', aliases })
    }
  }
]

GET.apiDoc = {
  description: 'Get the native and hopr addresses of the account associated with the node',
  tags: ['account'],
  operationId: 'accountGetAddress',
  parameters: [],
  responses: {
    '200': {
      description: 'Returns the native and hopr addresses of the account associated with the node',
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/Address'
          }
        }
      }
    }
  }
}
