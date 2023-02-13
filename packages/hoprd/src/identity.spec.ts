import { getIdentity, IdentityErrors, IdentityOptions } from './identity.js'
import { stringToU8a } from '@hoprnet/hopr-utils'
import { unlinkSync, existsSync } from 'fs'
import assert from 'assert'

describe('Identity', function () {
  const DUMMY_PATH = new URL('./hopr-test-identity', import.meta.url).pathname
  const DUMMY_PASSWORD = 'hopr-unit-test-password'
  const WRONG_DUMMY_PASSWORD = 'hopr-unit-test-wrong-password'
  const DUMMY_PRIVATE_KEY = stringToU8a('cd09f9293ffdd69be978032c533b6bcd02dfd5d937c987bedec3e28de07e0317')
  const DUMMY_PREFIXED_PRIVATE_KEY = stringToU8a('0xcd09f9293ffdd69be978032c533b6bcd02dfd5d937c987bedec3e28de07e0317')

  const mockIdentityOptions: IdentityOptions = {
    initialize: false,
    idPath: DUMMY_PATH,
    password: DUMMY_PASSWORD
  }

  const initializedMockIdentity: IdentityOptions = { ...mockIdentityOptions, initialize: true }
  const createInitializedMockIdentityWithPassword: (password: string) => IdentityOptions = (password) => ({
    ...initializedMockIdentity,
    password
  })

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
    it('receives a private key and stores it on a given path serialized', async () => {
      const testIdentity = await getIdentity({
        ...initializedMockIdentity,
        useWeakCrypto: true,
        privateKey: DUMMY_PRIVATE_KEY
      })

      const deserializedIdentity = await getIdentity({
        ...initializedMockIdentity,
        useWeakCrypto: true,
        privateKey: DUMMY_PRIVATE_KEY
      })

      assert(testIdentity.equals(deserializedIdentity))
    })
    it('receives a 0x-prefixed private key and stores it on a given path serialized', async () => {
      const testIdentity = await getIdentity({
        ...initializedMockIdentity,
        useWeakCrypto: true,
        privateKey: DUMMY_PREFIXED_PRIVATE_KEY
      })

      const deserializedIdentity = await getIdentity({
        ...initializedMockIdentity,
        useWeakCrypto: true,
        privateKey: DUMMY_PREFIXED_PRIVATE_KEY
      })

      assert(testIdentity.equals(deserializedIdentity))
    })
  })

  it('fail to load non-existing key', async function () {
    await assert.rejects(
      async () => {
        await getIdentity({
          ...mockIdentityOptions
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
