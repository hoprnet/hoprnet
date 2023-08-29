import request from 'supertest'
import sinon from 'sinon'
import chaiResponseValidator from 'chai-openapi-response-validator'
import chai, { expect } from 'chai'

import { ApplicationData, MessageInbox, hoprd_hoprd_initialize_crate } from '../../../../../lib/hoprd_hoprd.js'
hoprd_hoprd_initialize_crate()

import { createTestApiInstance } from '../../fixtures.js'

import type { Hopr } from '@hoprnet/hopr-core'

describe('POST /messages/pop', function () {
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

  it('should return nothing when inbox is empty', async function () {
    const tag = 112

    expect(await inbox.size(tag)).to.equal(0)

    const res = await request(service).post(`/api/v3/messages/pop`).send({ tag })
    expect(res.status).to.equal(404)
    expect(await inbox.size(tag)).to.equal(0)
  })

  it('should return a message when inbox is not empty', async function () {
    const tag = 112

    expect(await inbox.size(tag)).to.equal(0)
    let appData = new ApplicationData(tag, new TextEncoder().encode('hello world'))
    await inbox.push(appData)
    expect(await inbox.size(tag)).to.equal(1)

    const res = await request(service).post(`/api/v3/messages/pop`).send({ tag })
    expect(res.status).to.equal(200)
    expect(res.body.tag).to.equal(tag)
    expect(res.body.body).to.equal('hello world')
    expect(await inbox.size(tag)).to.equal(0)
  })

  it('should return nothing when other inbox is not empty', async function () {
    const tag = 112
    const otherTag = 1121

    expect(await inbox.size(tag)).to.equal(0)
    let appData = new ApplicationData(tag, new TextEncoder().encode('hello world'))
    await inbox.push(appData)
    expect(await inbox.size(tag)).to.equal(1)

    const res = await request(service).post(`/api/v3/messages/pop`).send({ tag: otherTag })
    expect(res.status).to.equal(404)
    expect(await inbox.size(tag)).to.equal(1)
  })
})
