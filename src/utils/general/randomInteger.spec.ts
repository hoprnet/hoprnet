import assert from 'assert'
import randomInteger from './randomInteger'

describe('testing random-number generator', function() {
    let ATTEMPTS = 100
    it(`should output values between '0' and '23'`, function() {
        let result = []
        for (let i = 0; i < ATTEMPTS; i++) {
            result.push(randomInteger(23))
        }

        assert(result.every(value => 0 <= value && value < 23))
    })

    it(`should output values between '31' and '61'`, function() {
        let result = []
        for (let i = 0; i < ATTEMPTS; i++) {
            result.push(randomInteger(31, 61))
        }

        assert(result.every(value => 31 <= value && value < 61))
    })

    it('should throw error for falsy interval input', function() {
        assert.throws(() => randomInteger(2, 1))

        assert.throws(() => randomInteger(Math.pow(2, 32)))

        assert.throws(() => randomInteger(-1))

        assert.throws(() => randomInteger(-1, -2))

    })
})
