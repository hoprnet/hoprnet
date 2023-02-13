import request from 'supertest'
import sinon from 'sinon'
import chaiResponseValidator from 'chai-openapi-response-validator'
import chai, { expect } from 'chai'

import { createToken, storeToken } from '../../token.js'

import { createAuthenticatedTestApiInstance, createMockDb } from './../fixtures.js'

import type { default as Hopr } from '@hoprnet/hopr-core'

describe('GET /token', function () {
  let node: Hopr
  let service: any

  before(async function () {
    node = sinon.fake() as any
    node.db = createMockDb()

    const loaded = await createAuthenticatedTestApiInstance(node)

    service = loaded.service

    // @ts-ignore ESM / CommonJS compatibility issue
    chai.use(chaiResponseValidator.default(loaded.api.apiDoc))
  })

  it('should fail with not found error when using superuser token', async function () {
    const res = await request(service).get('/api/v2/token').set('x-auth-token', 'superuser')
    expect(res.status).to.equal(404)
    expect(res).to.satisfyApiSpec
  })

  it('should fail with unauthenticated error when using no token', async function () {
    const res = await request(service).get('/api/v2/token')
    expect(res.status).to.equal(401)
    expect(res).to.satisfyApiSpec
  })

  it('should fail with unauthorized error when using token with missing capability', async function () {
    // create token with wrong capability
    const caps = [{ endpoint: 'tokensCreate' }]
    const token = await createToken(node.db, caps)
    await storeToken(node.db, token)

    const res = await request(service).get('/api/v2/token').set('x-auth-token', token.id)
    expect(res.status).to.equal(403)
    expect(res).to.satisfyApiSpec
  })

  it('should succeed when using token with correct capability', async function () {
    // create token with correct capability
    const caps = [{ endpoint: 'tokensGetToken' }]
    const token = await createToken(node.db, caps)
    await storeToken(node.db, token)

    const res = await request(service).get('/api/v2/token').set('x-auth-token', token.id)
    expect(res.status).to.equal(200)
    expect(res).to.satisfyApiSpec
  })
})
