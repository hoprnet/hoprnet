import request from 'supertest'
import sinon from 'sinon'
import chaiResponseValidator from 'chai-openapi-response-validator'
import chai, { expect } from 'chai'

import { ApplicationData, MessageInbox, hoprd_hoprd_initialize_crate } from '../../../../../lib/hoprd_hoprd.js'
hoprd_hoprd_initialize_crate()

import { createTestApiInstance, ALICE_PEER_ID } from '../../fixtures.js'

import type { Hopr } from '@hoprnet/hopr-core'

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

  it('should work when other inbox is not empty', async function () {
    const tag = 112
    const otherTag = 1121

    expect(await inbox.size(tag)).to.equal(0)
    let appData = new ApplicationData(tag, new TextEncoder().encode('hello world'))
    await inbox.push(appData)
    expect(await inbox.size(tag)).to.equal(1)

    const res = await request(service).delete(`/api/v3/messages`).query({ tag: otherTag })
    expect(res.status).to.equal(204)
    expect(await inbox.size(tag)).to.equal(1)
  })
})

describe('POST /messages', function () {
  let node: Hopr
  let service: any
  let inbox: MessageInbox

  before(async function () {
    node = sinon.fake() as any as Hopr
    node.sendMessage = sinon.fake.resolves('some ack')
    const loaded = await createTestApiInstance(node)

    service = loaded.service
    inbox = loaded.inbox

    // @ts-ignore ESM / CommonJS compatibility issue
    chai.use(chaiResponseValidator.default(loaded.api.apiDoc))
  })

  it('should work when parameters are correct', async function () {
    const tag = 112
    const body = 'hello world'
    const peerId = ALICE_PEER_ID.toString()
    const hops = 1

    expect(await inbox.size(tag)).to.equal(0)

    const res = await request(service).post(`/api/v3/messages`).send({ tag, body, peerId, hops })
    expect(res.status).to.equal(202)
    expect(await inbox.size(tag)).to.equal(0)
  })

  it('should not work when parameters are incorrect', async function () {
    const tag = 112
    const body = 'hello world'
    const peerId = ALICE_PEER_ID.toString()
    const hops = 1

    expect(await inbox.size(tag)).to.equal(0)

    const res1 = await request(service).post(`/api/v3/messages`).send({ tag, body, peerId, hops: 0 })
    expect(res1.status).to.equal(400)

    const res2 = await request(service).post(`/api/v3/messages`).send({ tag, body, peerId, hops: 4 })
    expect(res2.status).to.equal(400)

    const res3 = await request(service).post(`/api/v3/messages`).send({ tag: 70000, body, peerId, hops })
    expect(res3.status).to.equal(400)

    const res4 = await request(service).post(`/api/v3/messages`).send({ tag: -1, body, peerId, hops })
    expect(res4.status).to.equal(400)

    const res5 = await request(service).post(`/api/v3/messages`).send({ tag, body, peerId: 'hello peer id', hops })
    expect(res5.status).to.equal(400)
  })
})
