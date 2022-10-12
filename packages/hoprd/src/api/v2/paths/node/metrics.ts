import type { Operation } from 'express-openapi'
import { STATUS_CODES } from '../../utils.js'
import { gather_all_metrics } from '@hoprnet/hopr-utils'

const GET: Operation = [
  (_, res, _next) => {
    try {
      const metrics = gather_all_metrics();
      return res.status(200).send(metrics)
    } catch (err) {
      return res
        .status(422)
        .send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: err instanceof Error ? err.message : 'Unknown error' })
    }
  }
]

GET.apiDoc = {
  description: 'Retrieve Prometheus metrics from the running node.',
  tags: ['Node'],
  operationId: 'nodeGetMetrics',
  responses: {
    '200': {
      description: 'Returns the encoded serialized metrics.',
      content: {
        'text/plain; version=0.0.4': {
          schema: {
            type: 'string',
            description: 'Prometheus metrics',
            example: '1.83.5'
          }
        }
      }
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

export default { GET }
