import { getIdentity } from './identity'
import { unlinkSync, existsSync } from 'fs'
import { resolve } from 'path'
import assert from 'assert'

describe('Identity', function () {
  const DUMMY_PATH = resolve(__dirname, './hopr-test-identity')
  const DUMMY_PASSWORD = 'hopr-unit-test-password'
  const WRONG_DUMMY_PASSWORD = 'hopr-unit-test-wrong-password'

  before(function () {
    if (existsSync(DUMMY_PATH)) {
      unlinkSync(DUMMY_PATH)
    }
  })

  after(function () {
    unlinkSync(DUMMY_PATH)
  })

  it('fail to load non-existing key', async function () {
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
  })

  it('fail to load non-existing key', async function () {
    // Store dummy identity
    await getIdentity({
      initialize: true,
      idPath: DUMMY_PATH,
      password: DUMMY_PASSWORD,
      useWeakCrypto: true
    })

    let rejectsWithWrongPassword: boolean
    try {
      await getIdentity({
        initialize: true,
        idPath: DUMMY_PATH,
        password: WRONG_DUMMY_PASSWORD,
        useWeakCrypto: true
      })
      rejectsWithWrongPassword = false
    } catch {
      rejectsWithWrongPassword = true
    }

    assert(rejectsWithWrongPassword)
  })

  it('fail to unintentionally load weakly encrypted secret', async function () {
    // Store dummy identity
    await getIdentity({
      initialize: true,
      idPath: DUMMY_PATH,
      password: DUMMY_PASSWORD,
      useWeakCrypto: true
    })

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

  it('store and restore identity', async function () {
    const testIdentity = await getIdentity({
      initialize: true,
      idPath: DUMMY_PATH,
      password: DUMMY_PASSWORD,
      useWeakCrypto: true
    })

    const deserializedIdentity = await getIdentity({
      initialize: false,
      idPath: DUMMY_PATH,
      password: DUMMY_PASSWORD,
      useWeakCrypto: true
    })

    assert(testIdentity.equals(deserializedIdentity))
  })
})
