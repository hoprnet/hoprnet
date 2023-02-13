import { setTimeout } from 'timers/promises'
import sinon from 'sinon'
import chai, { expect } from 'chai'
import chaiAsPromised from 'chai-as-promised'

import {
  authenticateToken,
  authorizeToken,
  createToken,
  storeToken,
  deleteToken,
  validateTokenCapabilities
} from './token.js'
import { createMockDb } from './v2/fixtures.js'

import type { default as Hopr } from '@hoprnet/hopr-core'
import type { Capability } from './token.js'

chai.use(chaiAsPromised)

describe('authentication token', function () {
  let node: Hopr

  before(async function () {
    node = sinon.fake() as any
    node.db = createMockDb()
  })

  it('should be created if parameters are valid', async function () {
    const caps: Array<Capability> = [{ endpoint: 'tokensCreate' }]
    expect(await createToken(node.db, caps)).to.have.a.property('id')
  })

  it('should not be created if parameters are invalid', async function () {
    const caps: Array<Capability> = [{ endpoint: 'tokensCreate2' }]
    expect(createToken(node.db, caps)).to.eventually.rejectedWith('invalid token capabilities')
  })

  it('should be created but not stored in the database', async function () {
    const caps: Array<Capability> = [{ endpoint: 'tokensCreate' }]
    const token = await createToken(node.db, caps)
    expect(token).to.have.a.property('id')
    expect(token.id).to.not.be.undefined

    expect(await authenticateToken(node.db, token.id)).to.be.undefined
  })

  it('should be stored', async function () {
    const caps: Array<Capability> = [{ endpoint: 'tokensCreate' }]
    const token = await createToken(node.db, caps)

    expect(storeToken(node.db, token)).to.eventually.be.fulfilled
    expect(await authenticateToken(node.db, token.id)).deep.equal(token)
  })

  it('should be deleted if exists', async function () {
    const caps: Array<Capability> = [{ endpoint: 'tokensCreate' }]
    const token = await createToken(node.db, caps)
    await storeToken(node.db, token)
    await deleteToken(node.db, token.id)

    expect(await authenticateToken(node.db, token.id)).to.be.undefined
  })

  it('should not fail to be deleted if id is empty', async function () {
    expect(deleteToken(node.db, '')).to.eventually.be.fulfilled
  })
})

describe('authentication token capabilities', function () {
  it('should validate if correct - one endpoint', async function () {
    const caps: Array<Capability> = [{ endpoint: 'tokensCreate' }]
    expect(validateTokenCapabilities(caps)).to.be.true
  })

  it('should validate if correct - two endpoint', async function () {
    const caps: Array<Capability> = [{ endpoint: 'tokensCreate' }, { endpoint: 'tokensGetToken' }]
    expect(validateTokenCapabilities(caps)).to.be.true
  })

  it('should validate if correct - two endpoints with limits', async function () {
    const caps: Array<Capability> = [
      { endpoint: 'tokensCreate', limits: [{ type: 'calls', conditions: { max: 1 } }] },
      { endpoint: 'tokensGetToken' }
    ]
    expect(validateTokenCapabilities(caps)).to.be.true
  })

  it('should not validate - empty list', async function () {
    const caps: Array<Capability> = []
    expect(validateTokenCapabilities(caps)).to.be.false
  })

  it('should not validate - one endpoint (unknown)', async function () {
    const caps: Array<Capability> = [{ endpoint: 'tokensCreate2' }]
    expect(validateTokenCapabilities(caps)).to.be.false
  })

  it('should not validate - two endpoints (one unknown)', async function () {
    const caps: Array<Capability> = [{ endpoint: 'tokensCreate' }, { endpoint: 'tokensGetToken2' }]
    expect(validateTokenCapabilities(caps)).to.be.false
  })

  it('should not validate - two endpoints (one with wrong limits - max)', async function () {
    const caps: Array<Capability> = [
      { endpoint: 'tokensCreate', limits: [{ type: 'calls', conditions: { max: 1 } }] },
      { endpoint: 'tokensGetToken', limits: [{ type: 'calls', conditions: { max: 0 } }] }
    ]
    expect(validateTokenCapabilities(caps)).to.be.false
  })

  it('should not validate - two endpoints (one with wrong limits - type)', async function () {
    const caps: Array<Capability> = [
      { endpoint: 'tokensCreate', limits: [{ type: 'calls2', conditions: { max: 1 } }] },
      { endpoint: 'tokensGetToken', limits: [{ type: 'calls', conditions: { max: 1 } }] }
    ]
    expect(validateTokenCapabilities(caps)).to.be.false
  })
})

describe('authentication token authorization', function () {
  let node: Hopr

  before(async function () {
    node = sinon.fake() as any
    node.db = createMockDb()
  })

  it('should succeed if lifetime is unset', async function () {
    const caps: Array<Capability> = [{ endpoint: 'tokensGetToken' }]
    let token = await createToken(node.db, caps)
    await storeToken(node.db, token)

    token = await authenticateToken(node.db, token.id)
    expect(token).to.not.have.a.property('valid_until')
  })

  it('should succeed if lifetime is still valid', async function () {
    const caps: Array<Capability> = [{ endpoint: 'tokensGetToken' }]
    const lifetime = 1000
    let token = await createToken(node.db, caps, '', lifetime)
    await storeToken(node.db, token)

    token = await authenticateToken(node.db, token.id)
    expect(token).to.have.a.property('valid_until')
  })

  it('should fail if lifetime has passed', async function () {
    const caps: Array<Capability> = [{ endpoint: 'tokensGetToken' }]
    const lifetime = 1
    let token = await createToken(node.db, caps, '', lifetime)
    await storeToken(node.db, token)

    await setTimeout(1001)

    token = await authenticateToken(node.db, token.id)
    expect(token).to.be.undefined
  })

  it('should update calls used counter and eventually fail', async function () {
    const caps: Array<Capability> = [
      { endpoint: 'tokensGetToken', limits: [{ type: 'calls', conditions: { max: 2 } }] }
    ]
    let token = await createToken(node.db, caps)
    await storeToken(node.db, token)

    token = await authenticateToken(node.db, token.id)
    expect(token.capabilities[0].limits[0]).to.not.include({ used: 0 })

    expect(authorizeToken(node.db, token, 'tokensGetToken')).to.eventually.be.true
    token = await authenticateToken(node.db, token.id)
    expect(token.capabilities[0].limits[0]).to.include({ used: 1 })

    expect(authorizeToken(node.db, token, 'tokensGetToken')).to.eventually.be.true
    token = await authenticateToken(node.db, token.id)
    expect(token.capabilities[0].limits[0]).to.include({ used: 2 })

    expect(authorizeToken(node.db, token, 'tokensGetToken')).to.eventually.be.false
    token = await authenticateToken(node.db, token.id)
    expect(token.capabilities[0].limits[0]).to.include({ used: 2 })
  })
})
