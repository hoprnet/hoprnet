import assert from 'assert'

import { satisfies } from './semver.js'

describe('test semver', async function () {
  it('satisfies handles correct version', async function () {
    assert(satisfies('1.1.0', '1'))
  })

  it('satisfies handles incorrect version', async function () {
    assert(!satisfies('1.1.0', '2'))
  })
})
