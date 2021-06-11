import { getIdentity } from './identity'
import { unlinkSync, existsSync } from 'fs'
import { resolve } from 'path'
import assert from 'assert'

describe('identity generation and serialization', function () {
  const DUMMY_PATH = resolve(__dirname, './hopr-test-identity')
  const DUMMY_PASSWORD = 'hopr-unit-test-password'

  before(function () {
    if (existsSync(DUMMY_PATH)) {
      unlinkSync(DUMMY_PATH)
    }
  })

  after(function () {
    unlinkSync(DUMMY_PATH)
  })
  it('create identity', async function () {
    let rejectsWhenNotExisting: boolean
    try {
      await getIdentity({
        initialize: false,
        idPath: DUMMY_PATH,
        password: DUMMY_PASSWORD
      })
      rejectsWhenNotExisting = false
    } catch {
      rejectsWhenNotExisting = true
    }
    assert(rejectsWhenNotExisting)

    const testIdentity = await getIdentity({
      initialize: true,
      idPath: DUMMY_PATH,
      password: DUMMY_PASSWORD,
      dev: true
    })

    const deserializedIdentity = await getIdentity({
      initialize: false,
      idPath: DUMMY_PATH,
      password: DUMMY_PASSWORD,
      dev: true
    })

    assert(testIdentity.equals(deserializedIdentity))

    let rejectsWhenUsingDevSecret: boolean
    try {
      await getIdentity({
        initialize: false,
        idPath: DUMMY_PATH,
        password: DUMMY_PASSWORD
      })
      rejectsWhenUsingDevSecret = false
    } catch {
      rejectsWhenUsingDevSecret = true
    }

    assert(rejectsWhenUsingDevSecret)
  })
})
