import assert from 'assert'
import { durations, isExpired } from './time.js'

describe('test time', function () {
  context('durations', function () {
    it('should be 1 second', function () {
      assert(durations.seconds(1) == 1e3)
    })

    it('should be 2 seconds', function () {
      assert(durations.seconds(2) == 2e3)
    })

    it('should be 1 minute', function () {
      assert(durations.minutes(1) == 1e3 * 60)
    })

    it('should be 2 minutes', function () {
      assert(durations.minutes(2) == 2e3 * 60)
    })

    it('should be 1 hour', function () {
      assert(durations.hours(1) == 1e3 * 60 * 60)
    })

    it('should be 2 hours', function () {
      assert(durations.hours(2) == 2e3 * 60 * 60)
    })

    it('should be 1 day', function () {
      assert(durations.days(1) == 1e3 * 60 * 60 * 24)
    })

    it('should be 2 days', function () {
      assert(durations.days(2) == 2e3 * 60 * 60 * 24)
    })
  })

  context('isExpired', function () {
    const now = 10
    const TTL = 1

    it('should not be expired', function () {
      assert(!isExpired(now, now, TTL))
      assert(!isExpired(now + 2, now, TTL))
    })

    it('should be expired', function () {
      assert(isExpired(now - 2, now, TTL))
    })
  })
})
