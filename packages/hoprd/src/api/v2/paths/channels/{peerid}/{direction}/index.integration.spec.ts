import assert from 'assert'
import sinon from 'sinon'
import request from 'supertest'
import chaiResponseValidator from 'chai-openapi-response-validator'
import chai, { expect } from 'chai'
import { STATUS_CODES } from '../../../../utils.js'
import {
  createTestApiInstance,
  ALICE_PEER_ID,
  ALICE_NATIVE_ADDR,
  BOB_PEER_ID,
  INVALID_PEER_ID, channelEntryCreateMock
} from '../../../../fixtures.js'
import { getChannel, closeChannel } from './index.js'
import { formatIncomingChannel, formatOutgoingChannel } from '../../index.js'

let node = sinon.fake() as any
node.getId = sinon.fake.returns(ALICE_PEER_ID)
node.getEthereumAddress = sinon.fake.returns(ALICE_NATIVE_ADDR)
const outgoingMock = channelEntryCreateMock()
const incomingMock = channelEntryCreateMock()

describe('getChannel', function () {
  it('should return no channels', async function () {
    assert.rejects(
      () => {
        return getChannel(node, BOB_PEER_ID.toString(), 'incoming')
      },
      (err: Error) => {
        return err.message.includes(STATUS_CODES.CHANNEL_NOT_FOUND)
      }
    )
    assert.rejects(
      () => {
        return getChannel(node, BOB_PEER_ID.toString(), 'outgoing')
      },
      (err: Error) => {
        return err.message.includes(STATUS_CODES.CHANNEL_NOT_FOUND)
      }
    )
  })

  it('should return outgoing channel', async function () {
    node.getChannel = sinon.stub()
    node.getChannel.withArgs(ALICE_PEER_ID, BOB_PEER_ID).resolves(outgoingMock)

    const outgoing = await getChannel(node, BOB_PEER_ID.toString(), 'outgoing')
    assert.notEqual(outgoing, undefined)
    assert.deepEqual(outgoing, formatOutgoingChannel(outgoingMock))
  })

  it('should return outgoing and incoming channels', async function () {
    node.getChannel = sinon.stub()
    node.getChannel.withArgs(ALICE_PEER_ID, BOB_PEER_ID).resolves(outgoingMock)
    node.getChannel.withArgs(BOB_PEER_ID, ALICE_PEER_ID).resolves(incomingMock)

    const outgoing = await getChannel(node, BOB_PEER_ID.toString(), 'outgoing')
    assert.notEqual(outgoing, undefined)
    assert.deepEqual(outgoing, formatOutgoingChannel(outgoingMock))
    const incoming = await getChannel(node, BOB_PEER_ID.toString(), 'incoming')
    assert.notEqual(incoming, undefined)
    assert.deepEqual(incoming, formatIncomingChannel(incomingMock))
  })
})

describe('closeChannel', () => {
  it('should close channel', async () => {
    const expectedStatus = { channelStatus: 2, receipt: 'receipt', success: true }
    node.closeChannel = sinon.fake.returns({ status: expectedStatus.channelStatus, receipt: expectedStatus.receipt })

    const closureStatus = await closeChannel(node, ALICE_PEER_ID.toString(), 'outgoing')
    assert.deepEqual(closureStatus, expectedStatus)
  })

  it('should fail on invalid peerId', async () => {
    assert.rejects(() => closeChannel(node, INVALID_PEER_ID, 'outgoing'), Error(STATUS_CODES.INVALID_PEERID))
  })

  it('should fail when node call fails', async () => {
    node.closeChannel = sinon.fake.throws(Error('unknown error'))

    assert.deepEqual(await closeChannel(node, ALICE_PEER_ID.toString(), 'outgoing'), {
      success: false,
      reason: STATUS_CODES.UNKNOWN_FAILURE
    })
  })
})

describe('GET /channels/{peerId}/{direction}', () => {
  let service: any
  before(async function () {
    const loaded = await createTestApiInstance(node)

    service = loaded.service

    // @ts-ignore ESM / CommonJS compatibility issue
    chai.use(chaiResponseValidator.default(loaded.api.apiDoc))
  })

  it('should get outgoing channels', async () => {
    node.getChannel = sinon.stub()
    node.getChannel.withArgs(ALICE_PEER_ID, BOB_PEER_ID).resolves(outgoingMock)
    const res = await request(service).get(`/api/v2/channels/${BOB_PEER_ID.toString()}/outgoing`)
    expect(res.status).to.equal(200)
    expect(res).to.satisfyApiSpec
  })

  it('should get outgoing channels', async () => {
    node.getChannel = sinon.stub()
    node.getChannel.withArgs(BOB_PEER_ID, ALICE_PEER_ID).resolves(incomingMock)
    const res = await request(service).get(`/api/v2/channels/${BOB_PEER_ID.toString()}/incoming`)
    expect(res.status).to.equal(200)
    expect(res).to.satisfyApiSpec
  })

  it('should fail for unsupported param', async () => {
    node.getChannel = sinon.stub()
    const res = await request(service).get(`/api/v2/channels/${BOB_PEER_ID.toString()}/unsupported`)
    expect(res.status).to.equal(400)
    expect(res).to.satisfyApiSpec
  })
})

describe('DELETE /channels/{peerId}/{direction}', () => {
  let service: any
  before(async function () {
    const loaded = await createTestApiInstance(node)

    service = loaded.service

    // @ts-ignore ESM / CommonJS compatibility issue
    chai.use(chaiResponseValidator.default(loaded.api.apiDoc))
  })

  it('should close outgoing channels', async () => {
    const expectedStatus = { channelStatus: 2, receipt: 'receipt' }
    node.closeChannel = sinon.fake.returns({ status: expectedStatus.channelStatus, receipt: expectedStatus.receipt })

    const res = await request(service).delete(`/api/v2/channels/${BOB_PEER_ID.toString()}/outgoing`)
    expect(res.status).to.equal(200)
    expect(res).to.satisfyApiSpec
  })

  it('should fail while closing incoming channels', async () => {
    node.closeChannel = sinon.fake.throws('unknown error')
    const res = await request(service).delete(`/api/v2/channels/${BOB_PEER_ID.toString()}/incoming`)
    expect(res.status).to.equal(422)
    expect(res).to.satisfyApiSpec
  })

  it('should fail for unsupported param', async () => {
    node.closeChannel = sinon.fake.throws('unknown error')
    const res = await request(service).delete(`/api/v2/channels/${BOB_PEER_ID.toString()}/unsupported`)
    expect(res.status).to.equal(400)
    expect(res).to.satisfyApiSpec
  })
})
