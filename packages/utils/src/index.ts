export * from './async/index.js'
export * from './types.js'
export * from './process/index.js'
export * from './u8a/index.js'

export * from './http.js'

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
  get_package_version,
  HoprLibConfig,
  HoprdConfig,
  SmartContractConfig,
  TagBloomFilter,
  TicketStatistics,
  WasmVecAccountEntry,
  get_contract_data,
  resolve_network,
  Hopr,
  peer_metadata_protocol_version_name,
  WasmHealth,
  HoprdPersistentDatabase
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
