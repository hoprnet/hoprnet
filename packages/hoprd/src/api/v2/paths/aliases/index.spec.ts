import { STATUS_CODES } from '../../utils'
import assert from 'assert'
import { getAliases, setAlias } from '.'
import { createTestMocks, INVALID_PEER_ID, ALICE_PEER_ID } from '../../fixtures'

const ALIAS = 'SOME_ALIAS'

describe('getAliases', () => {
  const mocks = createTestMocks()
  setAlias(mocks, ALIAS, ALICE_PEER_ID.toB58String())

  it('should successfuly get aliases', () => {
    const aliases = getAliases(mocks.getState())
    assert.deepEqual(aliases, { [ALIAS]: ALICE_PEER_ID.toB58String() })
  })
})

describe('setAlias', function () {
  const mocks = createTestMocks()

  it('should set alias successfuly', function () {
    setAlias(mocks, ALIAS, ALICE_PEER_ID.toB58String())
    assert.equal(mocks.getState().aliases.size, 1)
    assert.equal(mocks.getState().aliases.get(ALIAS).toB58String(), ALICE_PEER_ID.toB58String())
  })

  it('should throw error on invalid peerId', () => {
    assert.throws(
      () => setAlias(mocks, ALIAS, INVALID_PEER_ID),
      (err: Error) => {
        return err.message.includes(STATUS_CODES.INVALID_PEERID)
      }
    )
  })
})
