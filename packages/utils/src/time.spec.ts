import assert from 'assert'
import {durations} from './time'

describe('test time', function () {
  context('durations', function () {
    it('should be 1 second', function () {
      assert(durations.seconds(1) === 1e3, 'check durations.seconds')
    })
    it('should be 2 seconds', function () {
      assert(durations.seconds(2) === 2e3, 'check durations.seconds')
    })

    it('should be 1 minute', function () {
      assert(durations.minutes(1) === 1e3 * 60, 'check durations.minutes')
    })
    it('should be 2 minutes', function () {
      assert(durations.minutes(2) === 2e3 * 60, 'check durations.minutes')
    })

    it('should be 1 hour', function () {
      assert(durations.hours(1) === 1e3 * 60 * 60, 'check durations.hours')
    })
    it('should be 2 hours', function () {
      assert(durations.hours(2) === 2e3 * 60 * 60, 'check durations.hours')
    })

    it('should be 1 day', function () {
      assert(durations.days(1) === 1e3 * 60 * 60 * 24, 'check durations.days')
    })
    it('should be 2 days', function () {
      assert(durations.days(2) === 2e3 * 60 * 60 * 24, 'check durations.days')
    })
  })
})
