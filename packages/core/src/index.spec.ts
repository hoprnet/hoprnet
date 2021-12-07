import { connectorMock } from '@hoprnet/hopr-core-ethereum'
import { dbMock, debug } from '@hoprnet/hopr-utils'
import chai, { expect } from 'chai'
import chaiAsPromised from 'chai-as-promised'
import PeerId from 'peer-id'
import sinon from 'sinon'
import Hopr, { sampleOptions } from '.'

chai.use(chaiAsPromised)
const log = debug('hopr-core:test:index')

describe('hopr core (instance)', async function () {
  let peerId: PeerId
  beforeEach(async function () {
    peerId = await PeerId.create({ keyType: 'secp256k1', bits: 256 })
  })

  afterEach(function () {
    sinon.restore()
  })

  it('should be able to start a hopr node instance without crashing', async function () {
    this.timeout(5000)
    log('Creating hopr node...')
    const node = new Hopr(peerId, dbMock, connectorMock, sampleOptions)
    log('Node created with Id', node.getId().toB58String())
    expect(node instanceof Hopr)
    log('Starting node')
    await node.start()
    return expect(
      node.stop()
    ).to.not.eventually.rejected
  })
})
