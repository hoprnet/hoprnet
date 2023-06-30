import request from 'supertest'
import sinon from 'sinon'
import chaiResponseValidator from 'chai-openapi-response-validator'
import chai, { expect } from 'chai'

import { authenticateToken, createToken, storeToken } from '../../../token.js'

import { createAuthenticatedTestApiInstance } from '../../fixtures.js'

import type { default as Hopr } from '@hoprnet/hopr-core'
import type { Token } from './../../../token.js'
import { LevelDb } from '@hoprnet/hopr-utils'
import { Database, PublicKey } from '@hoprnet/hopr-core/lib/core_hopr.js'

describe('DELETE /tokens/{id}', function () {
  let node: Hopr
  let service: any
  let token: Token

  before(async function () {
    node = sinon.fake() as any
    let db = new LevelDb()
    await db.backend.open()
    node.db = new Database(db, PublicKey.from_peerid_str('16Uiu2HAmM9KAPaXA4eAz58Q7Eb3LEkDvLarU4utkyLwDeEK6vM5m'))

    const loaded = await createAuthenticatedTestApiInstance(node)
    service = loaded.service

    // @ts-ignore ESM / CommonJS compatibility issue
    chai.use(chaiResponseValidator.default(loaded.api.apiDoc))
  })

  beforeEach(async function () {
    // test token which should be deleted
    const caps = [{ endpoint: 'tokensCreate' }]
    token = await createToken(node.db, undefined, caps)
    await storeToken(node.db, token)
  })

  it('should fail with not found error when using superuser token but incorrect token id', async function () {
    const res = await request(service).delete(`/api/v2/tokens/${token.id}wrong`).set('x-auth-token', 'superuser')
    expect(res.status).to.equal(404)
    expect(res).to.satisfyApiSpec
  })

  it('should succeed when using superuser token', async function () {
    const res = await request(service).delete(`/api/v2/tokens/${token.id}`).set('x-auth-token', 'superuser')
    expect(res.status).to.equal(204)
    expect(res).to.satisfyApiSpec

    const tokenInDb = await authenticateToken(node.db, token.id)
    expect(tokenInDb).to.be.undefined
  })

  it('should fail with unauthenticated error when using no token', async function () {
    const res = await request(service).delete(`/api/v2/tokens/${token.id}`)
    expect(res.status).to.equal(401)
    expect(res).to.satisfyApiSpec
  })

  it('should fail with unauthenticated error when using token with missing capability', async function () {
    // create token with wrong capability
    const caps = [{ endpoint: 'tokensCreate' }]
    const wrongToken = await createToken(node.db, undefined, caps)
    await storeToken(node.db, wrongToken)

    const res = await request(service).delete(`/api/v2/tokens/${token.id}`).set('x-auth-token', wrongToken.id)
    expect(res.status).to.equal(403)
    expect(res).to.satisfyApiSpec
  })

  it('should succeed when using token with correct capability', async function () {
    // create token with correct capability
    const caps = [{ endpoint: 'tokensDelete' }]
    const correctToken = await createToken(node.db, undefined, caps)
    await storeToken(node.db, correctToken)

    const res = await request(service).delete(`/api/v2/tokens/${token.id}`).set('x-auth-token', correctToken.id)
    expect(res.status).to.equal(204)
    expect(res).to.satisfyApiSpec

    const tokenInDb = await authenticateToken(node.db, token.id)
    expect(tokenInDb).to.be.undefined
  })
})
