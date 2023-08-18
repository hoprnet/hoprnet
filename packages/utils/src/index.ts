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

// Load `utils-misc` crate
import { utils_misc_initialize_crate } from '../lib/utils_misc.js'
utils_misc_initialize_crate()
export { get_package_version } from '../lib/utils_misc.js'

// Load `utils-metrics` crate
import { utils_metrics_initialize_crate } from '../lib/utils_metrics.js'
utils_metrics_initialize_crate()

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
  gather_all_metrics
} from '../lib/utils_metrics.js'

export type MetricCollector = () => string

let metricCollectors: MetricCollector[] = []
export { metricCollectors }

export function getMetricsCollectors(): MetricCollector[] {
  return metricCollectors
}
export function registerMetricsCollector(collector: MetricCollector) {
  metricCollectors.push(collector)
}
