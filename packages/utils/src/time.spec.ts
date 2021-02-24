import { expect } from 'chai'
import { durations, isExpired } from './time'

describe('test time', function () {
  context('durations', function () {
    it('should be 1 second', function () {
      expect(durations.seconds(1)).to.equal(1e3)
    })

    it('should be 2 seconds', function () {
      expect(durations.seconds(2)).to.equal(2e3)
    })

    it('should be 1 minute', function () {
      expect(durations.minutes(1)).to.equal(1e3 * 60)
    })

    it('should be 2 minutes', function () {
      expect(durations.minutes(2)).to.equal(2e3 * 60)
    })

    it('should be 1 hour', function () {
      expect(durations.hours(1)).to.equal(1e3 * 60 * 60)
    })

    it('should be 2 hours', function () {
      expect(durations.hours(2)).to.equal(2e3 * 60 * 60)
    })

    it('should be 1 day', function () {
      expect(durations.days(1)).to.equal(1e3 * 60 * 60 * 24)
    })

    it('should be 2 days', function () {
      expect(durations.days(2)).to.equal(2e3 * 60 * 60 * 24)
    })
  })

  context('isExpired', function () {
    const now = 10
    const TTL = 1

    it('should not be expired', function () {
      expect(isExpired(now, now, TTL)).to.be.false
      expect(isExpired(now + 2, now, TTL)).to.be.false
    })

    it('should be expired', function () {
      expect(isExpired(now - 2, now, TTL)).to.be.true
    })
  })
})
