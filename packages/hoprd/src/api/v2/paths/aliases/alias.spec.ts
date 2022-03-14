import { STATUS_CODES } from '../../utils'
import assert from 'assert'
import { removeAlias, getAlias } from './{alias}'
import { createTestMocks, ALICE_PEER_ID } from '../../fixtures'
import { setAlias } from '.'

const ALIAS = 'some_alias'

describe('removeAlias', function () {
  const mocks = createTestMocks()

  it('should remove alias successfuly', function () {
    setAlias(mocks, ALIAS, ALICE_PEER_ID.toB58String())
    removeAlias(mocks, ALIAS)
    assert.equal(mocks.getState().aliases.size, 0)
    assert.equal(mocks.getState().aliases.get('alias'), undefined)
  })
})

describe('getAlias', () => {
  const mocks = createTestMocks()
  setAlias(mocks, ALIAS, ALICE_PEER_ID.toB58String())

  it('should successfuly get alias', () => {
    const alias = getAlias(mocks.getState(), ALIAS)
    assert.equal(alias, ALICE_PEER_ID.toB58String())
  })

  it('should throw error on invalid peerId', () => {
    assert.throws(
      () => getAlias(mocks.getState(), 'alias2'),
      (err: Error) => {
        return err.message.includes(STATUS_CODES.PEERID_NOT_FOUND)
      }
    )
  })
})
