import { getIdentity, IdentityErrors, IdentityOptions } from './identity'
import { unlinkSync, existsSync } from 'fs'
import { resolve } from 'path'
import assert from 'assert'

describe('Identity', function () {
  const DUMMY_PATH = resolve(__dirname, './hopr-test-identity')
  const DUMMY_PASSWORD = 'hopr-unit-test-password'
  const WRONG_DUMMY_PASSWORD = 'hopr-unit-test-wrong-password'
  const INVALID_PRIVATE_KEY = 'invalid_hex_string'

  const mockIdentityOptions: IdentityOptions = {
    initialize: false,
    idPath: DUMMY_PATH,
    password: DUMMY_PASSWORD
  }

  const initializedMockIdentity: IdentityOptions = { ...mockIdentityOptions, initialize: true }
  const createInitializedMockIdentityWithPassword: (password: string) => IdentityOptions = (password) => ({ ...initializedMockIdentity, password })

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

  describe('Private Key', () => {
    it('fails to load a non-hexadecimal value as private key', async () => {
      await assert.rejects(
        async () => {
          await getIdentity({
            ...mockIdentityOptions,
            privateKey: INVALID_PRIVATE_KEY
          })
        },
        {
          name: 'Error',
          message: IdentityErrors.INVALID_PRIVATE_KEY_GIVEN
        }
      )
    })
  })

  it('fail to load non-existing key', async function () {
    await assert.rejects(
      async () => {
        await getIdentity({
          ...mockIdentityOptions,
        })
      },
      {
        name: 'Error',
        message: IdentityErrors.FAIL_TO_LOAD_IDENTITY
      }
    )
  })

  it('fail to load non-existing key', async function () {
    // Store dummy identity
    await getIdentity({
      ...initializedMockIdentity,
      useWeakCrypto: true
    })

    await assert.rejects(
      async () => {
        await getIdentity({
          ...createInitializedMockIdentityWithPassword(WRONG_DUMMY_PASSWORD),
          useWeakCrypto: true
        })
      },
      {
        name: 'Error',
        message: IdentityErrors.WRONG_PASSPHRASE
      }
    )
  })

  it('fail to unintentionally load weakly encrypted secret', async function () {
    // Store dummy development identity
    await getIdentity({
      ...initializedMockIdentity,
      useWeakCrypto: true
    })

    await assert.rejects(
      async () => {
        await getIdentity({
          ...initializedMockIdentity
        })
      },
      {
        name: 'Error',
        message: IdentityErrors.WRONG_USAGE_OF_WEAK_CRYPTO
      }
    )
  })

  it('fail on empty password', async function () {
    await assert.rejects(
      async () => {
        await getIdentity({
          ...createInitializedMockIdentityWithPassword('')
        })
      },
      {
        name: 'Error',
        message: IdentityErrors.EMPTY_PASSWORD
      }
    )
  })

  it('store and restore identity', async function () {
    const testIdentity = await getIdentity({
      ...initializedMockIdentity,
      useWeakCrypto: true
    })

    const deserializedIdentity = await getIdentity({
      ...initializedMockIdentity,
      useWeakCrypto: true
    })

    assert(testIdentity.equals(deserializedIdentity))
  })
})
