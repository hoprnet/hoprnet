import assert from 'assert'
import { logger, errors } from 'ethers'
import { isErrorOutOfNativeFunds, isErrorOutOfHoprFunds, isErrorOutOfFunds } from '.'

describe('test out of funds', function () {
  it('should understand out of NATIVE funds error', function () {
    let error: any
    try {
      logger.throwError('insufficient funds for intrinsic transaction cost', errors.INSUFFICIENT_FUNDS)
    } catch (err) {
      error = err
    }

    assert(isErrorOutOfNativeFunds(error))
    assert.strictEqual(isErrorOutOfFunds(error), 'NATIVE')
  })

  it('should understand out of HOPR funds error', function () {
    let error: any
    try {
      logger.throwError('reverted', errors.CALL_EXCEPTION, {
        reason: 'SafeMath: subtraction overflow'
      })
    } catch (err) {
      error = err
    }

    assert(isErrorOutOfHoprFunds(error))
    assert.strictEqual(isErrorOutOfFunds(error), 'HOPR')
  })

  it('should not be an out of funds error', function () {
    let error: any
    try {
      logger.throwError('missed argument', errors.MISSING_ARGUMENT)
    } catch (err) {
      error = err
    }

    assert(!isErrorOutOfNativeFunds(error))
    assert(!isErrorOutOfHoprFunds(error))
    assert.strictEqual(isErrorOutOfFunds(error), false)
  })

  it('should not be an out of funds error on revert', function () {
    let error: any
    try {
      logger.throwError('reverted', errors.CALL_EXCEPTION)
    } catch (err) {
      error = err
    }

    assert(!isErrorOutOfNativeFunds(error))
    assert(!isErrorOutOfHoprFunds(error))
    assert.strictEqual(isErrorOutOfFunds(error), false)
  })
})
