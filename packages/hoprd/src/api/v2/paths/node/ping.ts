import { Operation } from 'express-openapi'
import { isError } from '../..'

export const ping = async ({ node, state, peerId }: { node: Hopr; state: State; peerId: string }) => {
  let validPeerId: PeerId
  try {
    validPeerId = checkPeerIdInput(peerId, state)
  } catch (err) {
    return new Error('invalidPeerId')
  }

  let pingResult: Awaited<ReturnType<Hopr['ping']>>
  let error: any

  try {
    pingResult = await node.ping(validPeerId)
  } catch (err) {
    error = err
  }

  if (pingResult.latency >= 0) {
    return { latency: pingResult.latency }
  }

  if (error && error.message) {
    return new Error('failure')
  }
  return new Error('timeout')
}

export const parameters = []

export const GET: Operation = [
  async (req, res, _next) => {
    const { stateOps, node } = req.context
    const { peerId } = req.query

    // @TODO: done by express?
    if (!peerId) {
      return res.status(400).send({ status: 'noPeerIdProvided' })
    }

    const pingRes = await ping({ peerId: peerId as string, stateOps, node })
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
    }
  }
}
