import request from 'supertest'
import sinon from 'sinon'
import chaiResponseValidator from 'chai-openapi-response-validator'
import chai, { expect } from 'chai'
import {
  createTestApiInstance,
  ALICE_PEER_ID,
  BOB_PEER_ID,
  CHARLIE_PEER_ID,
  ALICE_NATIVE_ADDR
} from '../../fixtures.js'
import { Balance, ChannelEntry, NativeBalance, PublicKey, UINT256, Hash, ChannelStatus } from '@hoprnet/hopr-utils'
import BN from 'bn.js'

// create ALICE node as self
let node = sinon.fake() as any
node.getId = sinon.fake.returns(ALICE_PEER_ID)
node.getEthereumAddress = sinon.fake.returns(ALICE_NATIVE_ADDR)
node.getNativeBalance = sinon.fake.returns(new NativeBalance(new BN(10)))
node.getBalance = sinon.fake.returns(new Balance(new BN(1)))

const incomingChannel = new ChannelEntry(
  PublicKey.fromPeerId(ALICE_PEER_ID),
  PublicKey.fromPeerId(BOB_PEER_ID),
  new Balance(new BN(1)),
  Hash.create(),
  new UINT256(new BN(1)),
  new UINT256(new BN(1)),
  ChannelStatus.Closed,
  new UINT256(new BN(1)),
  new UINT256(new BN(1))
)
const outgoingChannel = new ChannelEntry(
  PublicKey.fromPeerId(BOB_PEER_ID),
  PublicKey.fromPeerId(ALICE_PEER_ID),
  new Balance(new BN(2)),
  Hash.create(),
  new UINT256(new BN(2)),
  new UINT256(new BN(2)),
  ChannelStatus.Open,
  new UINT256(new BN(2)),
  new UINT256(new BN(2))
)
const otherChannel = new ChannelEntry(
  PublicKey.fromPeerId(BOB_PEER_ID),
  PublicKey.fromPeerId(CHARLIE_PEER_ID),
  new Balance(new BN(3)),
  Hash.create(),
  new UINT256(new BN(3)),
  new UINT256(new BN(3)),
  ChannelStatus.WaitingForCommitment,
  new UINT256(new BN(3)),
  new UINT256(new BN(3))
)
node.getAllChannels = sinon.fake.returns(Promise.resolve([incomingChannel, outgoingChannel, otherChannel]))

describe('GET /topology', function () {
  let service: any
  before(async function () {
    const loaded = await createTestApiInstance(node)

    service = loaded.service

    // @ts-ignore ESM / CommonJS compatibility issue
    chai.use(chaiResponseValidator.default(loaded.api.apiDoc))
  })

  it('should get channels list including closed', async function () {
    const res = await request(service).get('/api/v2/topology')
    expect(res.status).to.equal(200)
    expect(res).to.satisfyApiSpec
    expect(res.body.length).to.be.equal(3)
    expect(res.body[0].channelId).to.deep.equal(incomingChannel.getId().toHex())
    expect(res.body[1].channelId).to.deep.equal(outgoingChannel.getId().toHex())
    expect(res.body[2].channelId).to.deep.equal(otherChannel.getId().toHex())
  })
})
