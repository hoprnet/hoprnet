import type { Operation } from 'express-openapi'
import { STATUS_CODES } from '../../utils.js'
import { getHeapStatistics } from 'v8'
import { create_gauge, gather_all_metrics } from '@hoprnet/hopr-utils'

// Metrics
const metric_totalAllocHeap = create_gauge(
  'hoprd_gauge_nodejs_total_alloc_heap_bytes',
  'V8 allocated total heap size in bytes'
)
const metric_totalUsedHeap = create_gauge('hoprd_gauge_nodejs_total_used_heap_bytes', 'V8 used heap size in bytes')
const metric_totalAvailHeap = create_gauge(
  'hoprd_gauge_nodejs_total_available_heap_bytes',
  'V8 total available heap size in bytes'
)
const metric_activeCtxs = create_gauge(
  'hoprd_gauge_nodejs_num_native_contexts',
  'V8 number of active top-level native contexts'
)
const metric_detachedCtxs = create_gauge(
  'hoprd_gauge_nodejs_num_detached_contexts',
  'V8 number of detached contexts which are not GCd'
)

function recordNodeHeapStats() {
  const heapStats = getHeapStatistics()
  metric_totalAllocHeap.set(heapStats.total_heap_size)
  metric_totalUsedHeap.set(heapStats.used_heap_size)
  metric_totalAvailHeap.set(heapStats.total_available_size)
  metric_activeCtxs.set(heapStats.number_of_native_contexts)
  metric_detachedCtxs.set(heapStats.number_of_detached_contexts)
}

const GET: Operation = [
  (_, res, _next) => {
    try {
      recordNodeHeapStats()
      const metrics = gather_all_metrics()
      return res.status(200).type('text/plain; version=0.0.4').send(metrics)
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
            description: 'Prometheus metrics text format',
            example: 'basic_counter 30'
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
