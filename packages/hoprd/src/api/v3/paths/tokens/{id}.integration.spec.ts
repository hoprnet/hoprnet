import request from 'supertest'
import sinon from 'sinon'
import chaiResponseValidator from 'chai-openapi-response-validator'
import chai, { expect } from 'chai'
import { Hopr, HoprdPersistentDatabase } from '@hoprnet/hopr-utils'

import { authenticateToken, createToken, storeToken } from '../../../token.js'

import { createAuthenticatedTestApiInstance } from '../../fixtures.js'
import type { Token } from './../../../token.js'

describe('DELETE /tokens/{id}', function () {
  let node: Hopr
  let service: any
  let db: HoprdPersistentDatabase
  let token: Token

  before(async function () {
    node = sinon.fake() as any

    const loaded = await createAuthenticatedTestApiInstance(node)
    service = loaded.service
    db = loaded.db

    // @ts-ignore ESM / CommonJS compatibility issue
    chai.use(chaiResponseValidator.default(loaded.api.apiDoc))
  })

  beforeEach(async function () {
    // test token which should be deleted
    const caps = [{ endpoint: 'tokensCreate' }]
    token = await createToken(db, undefined, caps)
    await storeToken(db, token)
  })

  it('should fail with not found error when using superuser token but incorrect token id', async function () {
    const res = await request(service).delete(`/api/v3/tokens/${token.id}wrong`).set('x-auth-token', 'superuser')
    expect(res.status).to.equal(404)
    expect(res).to.satisfyApiSpec
  })

  it('should succeed when using superuser token', async function () {
    const res = await request(service).delete(`/api/v3/tokens/${token.id}`).set('x-auth-token', 'superuser')
    expect(res.status).to.equal(204)
    expect(res).to.satisfyApiSpec

    const tokenInDb = await authenticateToken(db, token.id)
    expect(tokenInDb).to.be.undefined
  })

  it('should fail with unauthenticated error when using no token', async function () {
    const res = await request(service).delete(`/api/v3/tokens/${token.id}`)
    expect(res.status).to.equal(401)
    expect(res).to.satisfyApiSpec
  })

  it('should fail with unauthenticated error when using token with missing capability', async function () {
    // create token with wrong capability
    const caps = [{ endpoint: 'tokensCreate' }]
    const wrongToken = await createToken(db, undefined, caps)
    await storeToken(db, wrongToken)

    const res = await request(service).delete(`/api/v3/tokens/${token.id}`).set('x-auth-token', wrongToken.id)
    expect(res.status).to.equal(403)
    expect(res).to.satisfyApiSpec
  })

  it('should succeed when using token with correct capability', async function () {
    // create token with correct capability
    const caps = [{ endpoint: 'tokensDelete' }]
    const correctToken = await createToken(db, undefined, caps)
    await storeToken(db, correctToken)

    const res = await request(service).delete(`/api/v3/tokens/${token.id}`).set('x-auth-token', correctToken.id)
    expect(res.status).to.equal(204)
    expect(res).to.satisfyApiSpec

    const tokenInDb = await authenticateToken(db, token.id)
    expect(tokenInDb).to.be.undefined
  })
})
