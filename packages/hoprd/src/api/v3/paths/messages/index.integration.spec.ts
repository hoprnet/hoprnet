import request from 'supertest'
import sinon from 'sinon'
import chaiResponseValidator from 'chai-openapi-response-validator'
import chai, { expect } from 'chai'

import { createTestApiInstance, ALICE_PEER_ID } from '../../fixtures.js'
import { ApplicationData, MessageInbox, hoprd_inbox_initialize_crate } from '../../../../../lib/hoprd_inbox.js'
hoprd_inbox_initialize_crate()

import type Hopr from '@hoprnet/hopr-core'

describe('DELETE /messages', function () {
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

  it('should work when inbox is empty', async function () {
    const tag = 112

    expect(await inbox.size(tag)).to.equal(0)

    const res = await request(service).delete(`/api/v3/messages`).query({ tag })
    expect(res.status).to.equal(204)
    expect(await inbox.size(tag)).to.equal(0)
  })

  it('should work when inbox is not empty', async function () {
    const tag = 112

    expect(await inbox.size(tag)).to.equal(0)
    let appData = new ApplicationData(tag, new TextEncoder().encode('hello world'))
    await inbox.push(appData)
    expect(await inbox.size(tag)).to.equal(1)

    const res = await request(service).delete(`/api/v3/messages`).query({ tag })
    expect(res.status).to.equal(204)
    expect(await inbox.size(tag)).to.equal(0)
  })
})

describe('POST /messages', function () {
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

  it('should work when parameters are correct', async function () {
    const tag = 112
    const body = 'hello world'
    const recipient = ALICE_PEER_ID
    const hops = 1

    expect(await inbox.size(tag)).to.equal(0)
    console.log(ALICE_PEER_ID)

    const res = await request(service).post(`/api/v3/messages`).send({ tag, body, recipient, hops })
    expect(res.status).to.equal(202)
    expect(await inbox.size(tag)).to.equal(1)
  })
})
