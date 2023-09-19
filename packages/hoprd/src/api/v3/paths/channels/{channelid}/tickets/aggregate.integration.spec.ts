import sinon from 'sinon'
import request from 'supertest'
import chaiResponseValidator from 'chai-openapi-response-validator'
import chai, { expect } from 'chai'

import { generate_channel_id } from '@hoprnet/hopr-utils'

import { STATUS_CODES } from '../../../../utils.js'
import { createTestApiInstance, ALICE_ETHEREUM_ADDR, BOB_ETHEREUM_ADDR } from './../../../../fixtures.js'

import type { Hopr } from '@hoprnet/hopr-core'

describe('POST /channels/{channelid}/tickets/aggregate', function () {
  let node: Hopr
  let service: any

  before(async function () {
    node = sinon.fake() as any

    const loaded = await createTestApiInstance(node)

    service = loaded.service

    // @ts-ignore ESM / CommonJS compatibility issue
    chai.use(chaiResponseValidator.default(loaded.api.apiDoc))
  })

  it('should fail if channel id is invalid', async function () {
    const res = await request(service).post(`/api/v3/channels/somechannelid/tickets/aggregate`).send()
    expect(res).to.satisfyApiSpec
    expect(res.body.status).to.equal(STATUS_CODES.INVALID_CHANNELID)
    expect(res.status).to.equal(400)
  })

  it('should fail if channel is not found', async function () {
    node.aggregateTickets = sinon.fake.rejects(new Error('Cannot aggregate tickets in non-existing channel'))
    let channelId = generate_channel_id(ALICE_ETHEREUM_ADDR, BOB_ETHEREUM_ADDR)
    const res = await request(service).post(`/api/v3/channels/${channelId.to_hex()}/tickets/aggregate`).send()
    expect(res).to.satisfyApiSpec
    expect(res.status).to.equal(404)
  })

  it('should fail if no tickets are found', async function () {
    node.aggregateTickets = sinon.fake.rejects(new Error('No tickets found in channel'))
    let channelId = generate_channel_id(ALICE_ETHEREUM_ADDR, BOB_ETHEREUM_ADDR)
    const res = await request(service).post(`/api/v3/channels/${channelId.to_hex()}/tickets/aggregate`).send()
    expect(res).to.satisfyApiSpec
    expect(res.body.status).to.equal(STATUS_CODES.TICKETS_NOT_FOUND)
    expect(res.status).to.equal(422)
  })

  it('should succeed if channel has tickets', async function () {
    node.aggregateTickets = sinon.fake.resolves()
    let channelId = generate_channel_id(ALICE_ETHEREUM_ADDR, BOB_ETHEREUM_ADDR)
    const res = await request(service).post(`/api/v3/channels/${channelId.to_hex()}/tickets/aggregate`).send()
    expect(res).to.satisfyApiSpec
    expect(res.status).to.equal(204)
  })
})
