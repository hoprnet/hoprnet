import sinon from 'sinon'
import request from 'supertest'
import chaiResponseValidator from 'chai-openapi-response-validator'
import chai, { expect } from 'chai'

import { generate_channel_id } from '@hoprnet/hopr-utils'

import { STATUS_CODES } from '../../../utils.js'
import { createTestApiInstance, ALICE_ETHEREUM_ADDR, BOB_ETHEREUM_ADDR } from './../../../fixtures.js'

import type { Hopr } from '@hoprnet/hopr-core'

describe('POST /channels/{channelid}/fund', function () {
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
    const res = await request(service).post(`/api/v3/channels/somechannelid/fund`).send({ amount: '1' })
    expect(res).to.satisfyApiSpec
    expect(res.body.status).to.equal(STATUS_CODES.INVALID_CHANNELID)
    expect(res.status).to.equal(400)
  })

  it('should fail if channel is not found', async function () {
    node.fundChannel = sinon.fake.rejects(new Error('Cannot fund non-existing channel'))
    let channelId = generate_channel_id(ALICE_ETHEREUM_ADDR, BOB_ETHEREUM_ADDR)
    const res = await request(service).post(`/api/v3/channels/${channelId.to_hex()}/fund`).send({ amount: '1' })
    expect(res).to.satisfyApiSpec
    expect(res.status).to.equal(404)
  })

  it('should fail if channel if amount is missing', async function () {
    let channelId = generate_channel_id(ALICE_ETHEREUM_ADDR, BOB_ETHEREUM_ADDR)
    const res = await request(service).post(`/api/v3/channels/${channelId.to_hex()}/fund`).send()
    expect(res).to.satisfyApiSpec
    expect(res.status).to.equal(400)
  })

  it('should fail if channel if amount is too small', async function () {
    node.fundChannel = sinon.fake.rejects(new Error('Amount must be more than 0'))
    let channelId = generate_channel_id(ALICE_ETHEREUM_ADDR, BOB_ETHEREUM_ADDR)
    const res = await request(service).post(`/api/v3/channels/${channelId.to_hex()}/fund`).send({ amount: '0' })
    console.log(res.body)
    expect(res).to.satisfyApiSpec
    expect(res.status).to.equal(422)
    expect(res.body.status).to.equal(STATUS_CODES.AMOUNT_TOO_SMALL)
  })

  it('should fail if safe has not enough balance', async function () {
    node.fundChannel = sinon.fake.rejects(new Error('Not enough balance'))
    let channelId = generate_channel_id(ALICE_ETHEREUM_ADDR, BOB_ETHEREUM_ADDR)
    const res = await request(service).post(`/api/v3/channels/${channelId.to_hex()}/fund`).send({ amount: '1' })
    expect(res).to.satisfyApiSpec
    expect(res.status).to.equal(422)
    expect(res.body.status).to.equal(STATUS_CODES.NOT_ENOUGH_BALANCE)
  })

  it('should fail if safe has not enough allowance', async function () {
    node.fundChannel = sinon.fake.rejects(new Error('Not enough allowance'))
    let channelId = generate_channel_id(ALICE_ETHEREUM_ADDR, BOB_ETHEREUM_ADDR)
    const res = await request(service).post(`/api/v3/channels/${channelId.to_hex()}/fund`).send({ amount: '1' })
    expect(res).to.satisfyApiSpec
    expect(res.status).to.equal(422)
    expect(res.body.status).to.equal(STATUS_CODES.NOT_ENOUGH_ALLOWANCE)
  })

  it('should fail if channel is not open', async function () {
    node.fundChannel = sinon.fake.rejects(new Error('not in status OPEN'))
    let channelId = generate_channel_id(ALICE_ETHEREUM_ADDR, BOB_ETHEREUM_ADDR)
    const res = await request(service).post(`/api/v3/channels/${channelId.to_hex()}/fund`).send({ amount: '1' })
    expect(res).to.satisfyApiSpec
    expect(res.status).to.equal(422)
    expect(res.body.status).to.equal(STATUS_CODES.CHANNEL_NOT_OPEN)
  })

  it('should succeed if channel is open', async function () {
    node.fundChannel = sinon.fake.resolves('myreceipt')
    let channelId = generate_channel_id(ALICE_ETHEREUM_ADDR, BOB_ETHEREUM_ADDR)
    const res = await request(service).post(`/api/v3/channels/${channelId.to_hex()}/fund`).send({ amount: '1' })
    expect(res).to.satisfyApiSpec
    expect(res.status).to.equal(200)
    expect(res.body.receipt).to.equal('myreceipt')
  })
})
