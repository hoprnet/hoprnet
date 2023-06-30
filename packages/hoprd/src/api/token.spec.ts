import { setTimeout } from 'timers/promises'
import sinon from 'sinon'
import chai from 'chai'
import chaiAsPromised from 'chai-as-promised'

import {
  authenticateToken,
  authorizeToken,
  createToken,
  storeToken,
  deleteToken,
  validateTokenCapabilities
} from './token.js'

import type { default as Hopr } from '@hoprnet/hopr-core'
import type { Capability } from './token.js'
import { Database, PublicKey, core_hopr_initialize_crate } from '../../../core/lib/core_hopr.js'
core_hopr_initialize_crate()
import { LevelDb } from '@hoprnet/hopr-utils'

chai.should()
chai.use(chaiAsPromised)

describe('authentication token', function () {
  let node: Hopr

  beforeEach(async function () {
    node = sinon.fake() as any
    let db = new LevelDb()
    await db.backend.open()
    node.db = new Database(db, PublicKey.from_peerid_str('16Uiu2HAmM9KAPaXA4eAz58Q7Eb3LEkDvLarU4utkyLwDeEK6vM5m'))
  })

  it('should be created if parameters are valid', async function () {
    const caps: Array<Capability> = [{ endpoint: 'tokensCreate' }]

    const promise = createToken(node.db, undefined, caps).should.eventually.have.property('id')

    return promise
  })

  it('should not be created if parameters are invalid', async function () {
    const caps: Array<Capability> = [{ endpoint: 'tokensCreate2' }]

    const promise = createToken(node.db, undefined, caps).should.be.rejectedWith('invalid token capabilities')

    return promise
  })

  it('should be created but not stored in the database', async function () {
    const caps: Array<Capability> = [{ endpoint: 'tokensCreate' }]
    const token = await createToken(node.db, undefined, caps)

    token.should.have.property('id')
    token.id.should.not.be.undefined

    const promise = authenticateToken(node.db, token.id).should.eventually.be.undefined

    return promise
  })

  it('should be stored', async function () {
    const caps: Array<Capability> = [{ endpoint: 'tokensCreate' }]
    const token = await createToken(node.db, undefined, caps)

    await storeToken(node.db, token)

    const promise = authenticateToken(node.db, token.id).should.eventually.be.deep.equal(token)

    return promise
  })

  it('should be deleted if exists', async function () {
    const caps: Array<Capability> = [{ endpoint: 'tokensCreate' }]
    const token = await createToken(node.db, undefined, caps)
    await storeToken(node.db, token)
    await deleteToken(node.db, token.id)

    const promise = authenticateToken(node.db, token.id).should.eventually.be.undefined

    return promise
  })

  it('should not fail to be deleted if id is empty', async function () {
    const promise = deleteToken(node.db, '').should.eventually.be.fulfilled

    return promise
  })

  it('should not be created if lifetime exceeds scopes lifetime', async function () {
    const caps: Array<Capability> = [{ endpoint: 'tokensGetToken' }]

    const scopeToken = await createToken(node.db, undefined, caps, '', 1000)

    // lifetime too long
    const promiseTooLong = createToken(node.db, scopeToken, caps, '', 9999).should.be.rejectedWith(
      'requested token lifetime not allowed'
    )
    // lifetime unlimited
    const promiseUnlimited = createToken(node.db, scopeToken, caps, '', undefined).should.be.rejectedWith(
      'requested token lifetime not allowed'
    )

    return Promise.all([promiseTooLong, promiseUnlimited])
  })

  it('should not be created if capabilities are not a subset of scope', async function () {
    const caps: Array<Capability> = [{ endpoint: 'tokensGetToken' }, { endpoint: 'messagesSendMessage' }]

    const scopeToken = await createToken(node.db, undefined, caps)

    // no common element
    const capsNoCommon: Array<Capability> = [{ endpoint: 'messagesSign' }]
    const promiseNoCommon = createToken(node.db, scopeToken, capsNoCommon).should.be.rejectedWith(
      'requested token capabilities not allowed'
    )

    // one common element, but also uncommon element
    const capsOneCommon: Array<Capability> = [{ endpoint: 'messagesSign' }, { endpoint: 'messagesSendMessage' }]
    const promiseOneCommon = createToken(node.db, scopeToken, capsOneCommon).should.be.rejectedWith(
      'requested token capabilities not allowed'
    )

    return Promise.all([promiseNoCommon, promiseOneCommon])
  })

  it('should be created if capabilities are a subset of scope', async function () {
    const caps: Array<Capability> = [{ endpoint: 'tokensGetToken' }, { endpoint: 'messagesSendMessage' }]

    const scopeToken = await createToken(node.db, undefined, caps)

    // partial subset
    const capsPartial: Array<Capability> = [{ endpoint: 'messagesSendMessage' }]
    const promisePartial = createToken(node.db, scopeToken, capsPartial).should.eventually.be.fulfilled

    // same caps
    const capsFull: Array<Capability> = [{ endpoint: 'tokensGetToken' }, { endpoint: 'messagesSendMessage' }]
    const promiseFull = createToken(node.db, scopeToken, capsFull).should.eventually.be.fulfilled

    return Promise.all([promisePartial, promiseFull])
  })
})

describe('authentication token capabilities', function () {
  it('should validate if correct - one endpoint', async function () {
    const caps: Array<Capability> = [{ endpoint: 'tokensCreate' }]

    validateTokenCapabilities(caps).should.be.true
  })

  it('should validate if correct - two endpoint', async function () {
    const caps: Array<Capability> = [{ endpoint: 'tokensCreate' }, { endpoint: 'tokensGetToken' }]

    validateTokenCapabilities(caps).should.be.true
  })

  it('should validate if correct - two endpoints with limits', async function () {
    const caps: Array<Capability> = [
      { endpoint: 'tokensCreate', limits: [{ type: 'calls', conditions: { max: 1 } }] },
      { endpoint: 'tokensGetToken' }
    ]

    validateTokenCapabilities(caps).should.be.true
  })

  it('should not validate - empty list', async function () {
    const caps: Array<Capability> = []

    validateTokenCapabilities(caps).should.be.false
  })

  it('should not validate - one endpoint (unknown)', async function () {
    const caps: Array<Capability> = [{ endpoint: 'tokensCreate2' }]
    validateTokenCapabilities(caps).should.be.false
  })

  it('should not validate - two endpoints (one unknown)', async function () {
    const caps: Array<Capability> = [{ endpoint: 'tokensCreate' }, { endpoint: 'tokensGetToken2' }]
    validateTokenCapabilities(caps).should.be.false
  })

  it('should not validate - two endpoints (one with wrong limits - max)', async function () {
    const caps: Array<Capability> = [
      { endpoint: 'tokensCreate', limits: [{ type: 'calls', conditions: { max: 1 } }] },
      { endpoint: 'tokensGetToken', limits: [{ type: 'calls', conditions: { max: 0 } }] }
    ]

    validateTokenCapabilities(caps).should.be.false
  })

  it('should not validate - two endpoints of the same name (one with wrong limits - max)', async function () {
    const caps: Array<Capability> = [
      { endpoint: 'tokensGetToken', limits: [{ type: 'calls', conditions: { max: 1 } }] },
      { endpoint: 'tokensGetToken', limits: [{ type: 'calls', conditions: { max: 0 } }] }
    ]

    validateTokenCapabilities(caps).should.be.false
  })

  it('should validate - two endpoints of the same name (both with correct limits - max)', async function () {
    const caps: Array<Capability> = [
      { endpoint: 'tokensGetToken', limits: [{ type: 'calls', conditions: { max: 1 } }] },
      { endpoint: 'tokensGetToken', limits: [{ type: 'calls', conditions: { max: 1 } }] }
    ]

    validateTokenCapabilities(caps).should.be.true
  })

  it('should not validate - two endpoints (one with wrong limits - type)', async function () {
    const caps: Array<any> = [
      { endpoint: 'tokensCreate', limits: [{ type: 'calls2', conditions: { max: 1 } }] },
      { endpoint: 'tokensGetToken', limits: [{ type: 'calls', conditions: { max: 1 } }] }
    ]

    validateTokenCapabilities(caps).should.be.false
  })
})

describe('authentication token authorization', function () {
  let node: Hopr

  before(async function () {
    node = sinon.fake() as any
    let db = new LevelDb()
    await db.backend.open()
    node.db = new Database(db, PublicKey.from_peerid_str('16Uiu2HAmM9KAPaXA4eAz58Q7Eb3LEkDvLarU4utkyLwDeEK6vM5m'))
  })

  it('should succeed if lifetime is unset', async function () {
    const caps: Array<Capability> = [{ endpoint: 'tokensGetToken' }]
    let token = await createToken(node.db, undefined, caps)
    await storeToken(node.db, token)

    const promise = authenticateToken(node.db, token.id).should.eventually.not.have.property('valid_until')

    return promise
  })

  it('should succeed if lifetime is still valid', async function () {
    const caps: Array<Capability> = [{ endpoint: 'tokensGetToken' }]
    const lifetime = 1000
    let token = await createToken(node.db, undefined, caps, '', lifetime)
    await storeToken(node.db, token)

    const promise = authenticateToken(node.db, token.id).should.eventually.have.property('valid_until')

    return promise
  })

  it('should fail if lifetime has passed', async function () {
    const caps: Array<Capability> = [{ endpoint: 'tokensGetToken' }]
    const lifetime = 1
    let token = await createToken(node.db, undefined, caps, '', lifetime)
    await storeToken(node.db, token)

    await setTimeout(1001)

    const promise = authenticateToken(node.db, token.id).should.eventually.be.undefined

    return promise
  })

  it('should update calls used counter and eventually fail', async function () {
    const caps: Array<Capability> = [
      { endpoint: 'tokensGetToken', limits: [{ type: 'calls', conditions: { max: 2 } }] }
    ]
    let token = await createToken(node.db, undefined, caps)
    let authorized: boolean
    await storeToken(node.db, token)

    token = await authenticateToken(node.db, token.id)
    token.capabilities[0].limits[0].should.not.include({ used: 0 })

    authorized = await authorizeToken(node.db, token, 'tokensGetToken')
    authorized.should.be.true
    token = await authenticateToken(node.db, token.id)
    token.capabilities[0].limits[0].should.include({ used: 1 })

    authorized = await authorizeToken(node.db, token, 'tokensGetToken')
    authorized.should.be.true
    token = await authenticateToken(node.db, token.id)
    token.capabilities[0].limits[0].should.include({ used: 2 })

    authorized = await authorizeToken(node.db, token, 'tokensGetToken')
    authorized.should.be.false
    token = await authenticateToken(node.db, token.id)
    token.capabilities[0].limits[0].should.include({ used: 2 })
  })

  it('should update calls used counter for used endpoint', async function () {
    const caps: Array<Capability> = [
      { endpoint: 'tokensGetToken', limits: [{ type: 'calls', conditions: { max: 2 } }] },
      { endpoint: 'messagesSendMessage', limits: [{ type: 'calls', conditions: { max: 2 } }] }
    ]
    let token = await createToken(node.db, undefined, caps)
    let authorized: boolean
    await storeToken(node.db, token)

    token = await authenticateToken(node.db, token.id)
    token.capabilities[0].limits[0].should.not.include({ used: 0 })
    token.capabilities[1].limits[0].should.not.include({ used: 0 })

    authorized = await authorizeToken(node.db, token, 'tokensGetToken')
    authorized.should.be.true
    token = await authenticateToken(node.db, token.id)
    token.capabilities.forEach((cap) => {
      if (cap.endpoint === 'tokenGetToken') {
        cap.limits[0].should.include({ used: 1 })
      } else if (cap.endpoint === 'messagesSendMessage') {
        cap.limits[0].should.not.own.property('used')
        // include({ used: 1 })
      }
    })

    authorized = await authorizeToken(node.db, token, 'tokensGetToken')
    authorized.should.be.true
    token = await authenticateToken(node.db, token.id)
    token.capabilities.forEach((cap) => {
      if (cap.endpoint === 'tokenGetToken') {
        cap.limits[0].should.include({ used: 2 })
      } else if (cap.endpoint === 'messagesSendMessage') {
        cap.limits[0].should.not.own.property('used')
      }
    })

    authorized = await authorizeToken(node.db, token, 'tokensGetToken')
    authorized.should.be.false
    token = await authenticateToken(node.db, token.id)
    token.capabilities.forEach((cap) => {
      if (cap.endpoint === 'tokenGetToken') {
        cap.limits[0].should.include({ used: 2 })
      } else if (cap.endpoint === 'messagesSendMessage') {
        cap.limits[0].should.not.own.property('used')
      }
    })
  })
})
