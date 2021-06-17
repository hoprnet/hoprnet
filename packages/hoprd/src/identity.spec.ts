import { getIdentity } from './identity'
import { unlinkSync, existsSync } from 'fs'
import { resolve } from 'path'
import assert from 'assert'

describe('Identity', function () {
  const DUMMY_PATH = resolve(__dirname, './hopr-test-identity')
  const DUMMY_PASSWORD = 'hopr-unit-test-password'
  const WRONG_DUMMY_PASSWORD = 'hopr-unit-test-wrong-password'

  beforeEach(function () {
    if (existsSync(DUMMY_PATH)) {
      unlinkSync(DUMMY_PATH)
    }
  })

  afterEach(function () {
    if (existsSync(DUMMY_PATH)) {
      unlinkSync(DUMMY_PATH)
    }
  })

  it('fail to load non-existing key', async function () {
    await assert.rejects(
      async () => {
        await getIdentity({
          initialize: false,
          idPath: DUMMY_PATH,
          password: DUMMY_PASSWORD
        })
      },
      {
        name: 'Error',
        message: 'Cannot load identity'
      }
    )
  })

  it('fail to load non-existing key', async function () {
    // Store dummy identity
    await getIdentity({
      initialize: true,
      idPath: DUMMY_PATH,
      password: DUMMY_PASSWORD,
      useWeakCrypto: true
    })

    await assert.rejects(
      async () => {
        await getIdentity({
          initialize: true,
          idPath: DUMMY_PATH,
          password: WRONG_DUMMY_PASSWORD,
          useWeakCrypto: true
        })
      },
      {
        name: 'Error',
        message: 'Key derivation failed - possibly wrong passphrase'
      }
    )
  })

  it('fail to unintentionally load weakly encrypted secret', async function () {
    // Store dummy development identity
    await getIdentity({
      initialize: true,
      idPath: DUMMY_PATH,
      password: DUMMY_PASSWORD,
      useWeakCrypto: true
    })

    await assert.rejects(
      async () => {
        await getIdentity({
          initialize: false,
          idPath: DUMMY_PATH,
          password: DUMMY_PASSWORD
        })
      },
      {
        name: 'Error',
        message: 'Attempting to use a development key while not being in development mode'
      }
    )
  })

  it('fail on empty password', async function () {
    await assert.rejects(
      async () => {
        await getIdentity({
          initialize: true,
          idPath: DUMMY_PATH,
          password: ''
        })
      },
      {
        name: 'Error',
        message: 'Password must not be empty'
      }
    )
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
