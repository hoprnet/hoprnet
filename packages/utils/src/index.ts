export * from './async/index.js'
export * from './collection/index.js'
export * from './libp2p/index.js'
export * from './math/index.js'
export * from './network/index.js'
export * from './process/index.js'
export * from './types.js'
export * from './u8a/index.js'
export * from './time.js'
export * from './constants.js'
export * from './db/index.js'
export * from './ethereum/index.js'
export * from './utils.js'

export {
  create_counter,
  SimpleCounter,
  create_multi_counter,
  MultiCounter,
  create_gauge,
  SimpleGauge,
  create_multi_gauge,
  MultiGauge,
  create_histogram,
  create_histogram_with_buckets,
  SimpleHistogram,
  create_multi_histogram,
  create_multi_histogram_with_buckets,
  MultiHistogram,
  SimpleTimer,
  merge_encoded_metrics,
  gather_all_metrics,
  get_package_version
} from '../../hoprd/lib/hoprd_hoprd.js'

export type MetricCollector = () => string

var metricCollectors: MetricCollector[]

function getMetricsCollectors(): MetricCollector[] {
  metricCollectors ??= []
  return metricCollectors
}
function registerMetricsCollector(collector: MetricCollector) {
  metricCollectors ??= []
  metricCollectors.push(collector)
}

export { metricCollectors, getMetricsCollectors, registerMetricsCollector }
