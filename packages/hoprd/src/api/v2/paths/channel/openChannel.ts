import { Operation } from 'express-openapi'
import { isError } from '../../../../commands/v2'
import { setAlias } from '../../../../commands/v2/logic/alias'

export const parameters = []

export const POST: Operation = [
  async (req, res, _next) => {
    const { commands } = req.context
    const { peerId, alias } = req.body

    // NOTE: probably express can or already is handling it automatically
    if (!peerId || !alias) {
      return res.status(400).send({ status: 'missingBodyfields' })
    }

    const aliases = setAlias({ alias, peerId, state: commands.state })
    if (isError(aliases)) {
      return res.status(404).send({ status: 'invalidPeerId' })
    } else {
      return res.status(200).send({ status: 'success', aliases })
    }
  }
]

POST.apiDoc = {
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
