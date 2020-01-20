import { PRG } from './prg'
import { randomBytes } from 'crypto'
import assert from 'assert'

describe('Hopr Polkadot', async function() {
  it('should create a digest', function() {
    const [key, iv] = [randomBytes(PRG.KEY_LENGTH), randomBytes(PRG.IV_LENGTH)]

    const prg = PRG.createPRG(key, iv)
    const digest = prg.digest(0, 500)

    const firstSlice = prg.digest(0, 32)
    assert.equal(firstSlice.length, 32, `check length`)
    assert.deepEqual(firstSlice, digest.slice(0, 32), `check that beginning is the same`)

    const secondSlice = prg.digest(123, 234)
    assert.equal(secondSlice.length, 234 - 123, `check size`)
    assert.deepEqual(secondSlice, digest.slice(123, 234), `check that slice somewhere in the middle is the same`)
    assert.deepEqual(PRG.createPRG(key, iv).digest(123, 234), prg.digest(123, 234), `check that slice somewhere in the middle is the same when computed by different methods`)

    assert.throws(() => prg.digest(234, 234), `should throw when start == end`)
    assert.throws(() => prg.digest(234, 233), `should throw when start > end`)
  })
})
