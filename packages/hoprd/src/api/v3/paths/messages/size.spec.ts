import request from 'supertest'
import sinon from 'sinon'
import chaiResponseValidator from 'chai-openapi-response-validator'
import chai, { expect } from 'chai'

import { ApplicationData, MessageInbox, hoprd_hoprd_initialize_crate } from '../../../../../lib/hoprd_hoprd.js'
hoprd_hoprd_initialize_crate()

import { createTestApiInstance } from '../../fixtures.js'

import type { Hopr } from '@hoprnet/hopr-core'

describe('GET /messages/size', function () {
  let node: Hopr
  let service: any
  let inbox: MessageInbox

  before(async function () {
    node = sinon.fake() as any as Hopr
    const loaded = await createTestApiInstance(node)

    service = loaded.service
    inbox = loaded.inbox

    // @ts-ignore ESM / CommonJS compatibility issue
    chai.use(chaiResponseValidator.default(loaded.api.apiDoc))
  })

  it('should return 0 when inbox is empty', async function () {
    const tag = 112

    const res = await request(service).get(`/api/v3/messages/size`).query({ tag })
    expect(res.status).to.equal(200)
    expect(res.body.size).to.equal(0)
  })

  it('should return 1 when inbox has 1 entry', async function () {
    const tag = 112

    let appData = new ApplicationData(tag, new TextEncoder().encode('hello world'))
    await inbox.push(appData)

    const res = await request(service).get(`/api/v3/messages/size`).query({ tag })
    expect(res.status).to.equal(200)
    expect(res.body.size).to.equal(1)
  })

  it('should return 0 when other inbox has 1 entry', async function () {
    const tag = 112
    const otherTag = 1121

    let appData = new ApplicationData(tag, new TextEncoder().encode('hello world'))
    await inbox.push(appData)

    const res = await request(service).get(`/api/v3/messages/size`).query({ tag: otherTag })
    expect(res.status).to.equal(200)
    expect(res.body.size).to.equal(0)
  })
})
