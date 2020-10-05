import assert from 'assert'
import { durations } from './time'

describe('test time - durations', () => {
  it('should be 1 second', () => {
    assert(durations.seconds(1) === 1e3, 'check durations.seconds')
  })
  it('should be 2 seconds', () => {
    assert(durations.seconds(2) === 2e3, 'check durations.seconds')
  })

  it('should be 1 minute', () => {
    assert(durations.minutes(1) === 1e3 * 60, 'check durations.minutes')
  })
  it('should be 2 minutes', () => {
    assert(durations.minutes(2) === 2e3 * 60, 'check durations.minutes')
  })

  it('should be 1 hour', () => {
    assert(durations.hours(1) === 1e3 * 60 * 60, 'check durations.hours')
  })
  it('should be 2 hours', () => {
    assert(durations.hours(2) === 2e3 * 60 * 60, 'check durations.hours')
  })

  it('should be 1 day', () => {
    assert(durations.days(1) === 1e3 * 60 * 60 * 24, 'check durations.days')
  })
  it('should be 2 days', () => {
    assert(durations.days(2) === 2e3 * 60 * 60 * 24, 'check durations.days')
  })
})
