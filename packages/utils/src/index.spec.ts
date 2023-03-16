import { create_histogram_with_buckets, gather_all_metrics } from './index.js'
import assert from 'assert'
import { setTimeout } from 'timers/promises'

describe('wasm modules', function () {
  it('test histogram timer', async function () {
    this.timeout(3.5e3)
    let histogram = create_histogram_with_buckets(
      'my_histogram',
      'test description',
      Float64Array.from([1.0, 2.0, 3.0, 4.0])
    )

    const timer = histogram.start_measure()
    await setTimeout(2500)
    histogram.record_measure(timer)

    let metrics = gather_all_metrics().encode()
    assert(metrics.includes('my_histogram_bucket{le="1"} 0'))
    assert(metrics.includes('my_histogram_bucket{le="2"} 0'))
    assert(metrics.includes('my_histogram_bucket{le="3"} 1'))
    assert(metrics.includes('my_histogram_bucket{le="4"} 1'))
  })
})
