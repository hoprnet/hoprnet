import assert from 'assert'
import { expandVars } from './utils.js'

describe('utils', () => {
  it('expands vars', async () => {
    assert(expandVars('simple string', {}) === 'simple string')
    assert(expandVars('simple ${foo}', { foo: 'bar' }) === 'simple bar')
    assert.throws(() => expandVars('simple string ${foo}', { not_bar: 1 }))
  })
})
