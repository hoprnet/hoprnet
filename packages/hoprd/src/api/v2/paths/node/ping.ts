import { Operation } from 'express-openapi'
import { isError } from '../../logic'
import { ping } from '../../logic/ping'

export const parameters = []

export const GET: Operation = [
  async (req, res, _next) => {
    const { state, node } = req.context
    const { peerId } = req.query

    if (!peerId) {
      return res.status(400).send({ status: 'noPeerIdProvided' })
    }

    const pingRes = await ping({ peerId: peerId as string, state, node })
    if (isError(pingRes)) {
      return res.status(pingRes.message === 'invalidPeerId' ? 400 : 500).send({ status: pingRes.message })
    } else {
      return res.status(200).send({ status: 'success', ...pingRes })
    }
  }
]

GET.apiDoc = {
  description: 'Pings another node to check its availability',
  tags: ['node'],
  operationId: 'ping',
  parameters: [
    {
      name: 'peerId',
      in: 'query',
      description: 'PeerId that we want to ping',
      required: true,
      schema: {
        type: 'string',
        example: '16Uiu2HAmRFjDov6sbcZeppbnNFFTdx5hFoBzr8csBgevtKUex8y9'
      }
    }
  ],
  responses: {
    '200': {
      description: 'Ping successful',
      content: {
        'application/json': {
          schema: {
            type: 'object',
            properties: {
              status: { type: 'string', example: 'success' },
              latency: {
                type: 'number',
                example: 10
              }
            }
          }
        }
      }
    },
    '400': {
      description: 'Invalid peerId',
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/StatusResponse'
          },
          example: { status: 'invalidPeerId' }
        }
      }
    },
  }
}