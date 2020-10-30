import {PRG} from './prg'
import {randomBytes} from 'crypto'
import assert from 'assert'
import {randomInteger} from '../'
import {u8aEquals} from '../u8a'

describe('Test Pseudo-Random Generator', async function () {
  it('should create a digest', function () {
    const [key, iv] = [randomBytes(PRG.KEY_LENGTH), randomBytes(PRG.IV_LENGTH)]

    const prg = PRG.createPRG(key, iv)
    const digest = prg.digest(0, 500)

    const firstSlice = prg.digest(0, 32)
    assert.equal(firstSlice.length, 32, `check length`)
    assert(u8aEquals(firstSlice, digest.slice(0, 32)), `check that beginning is the same`)

    const start = randomInteger(0, 250)
    const end = randomInteger(start, start + 251)
    const secondSlice = prg.digest(start, end)
    assert.equal(secondSlice.length, end - start, `check size`)
    assert(u8aEquals(secondSlice, digest.slice(start, end)), `check that slice somewhere in the middle is the same`)
    assert(
      u8aEquals(PRG.createPRG(key, iv).digest(start, end), prg.digest(start, end)),
      `check that slice somewhere in the middle is the same when computed by different methods`
    )

    assert.throws(() => prg.digest(234, 234), `should throw when start == end`)
    assert.throws(() => prg.digest(234, 233), `should throw when start > end`)
  })
})
