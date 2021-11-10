import LibP2P from 'libp2p'
import Hopr, { resolveEnvironment } from '@hoprnet/hopr-core'
import { debug, privKeyToPeerId } from '@hoprnet/hopr-utils'
import sinon from 'sinon'
import { generateNode, generateNodeOptions } from '.'

const log = debug('hopr:test:cover-traffic')

describe('cover-traffic daemon', async function () {
  const privateKey = '0xcb1e5d91d46eb54a477a7eefec9c87a1575e3e5384d38f990f19c09aa8ddd332'
  const options = await generateNodeOptions(resolveEnvironment('master-xdai'))
  const peerId = privKeyToPeerId(privateKey)

  let node: Hopr, libp2p: LibP2P
  beforeEach(function () {
    node = generateNode(peerId, options)
    libp2p = sinon.createStubInstance(LibP2P)
    sinon.stub(node.indexer, 'start').callsFake(() => Promise.resolve(log('indexer called for cover-traffic daemon')))
    sinon.stub(node.indexer, 'getPublicNodes').callsFake(() => { log('indexer called for public nodes'); return Promise.resolve([]) })
    sinon.stub(LibP2P, 'create').callsFake(() => { log('libp2p stub started'); return Promise.resolve(libp2p) })
  })
  afterEach(function () {
    sinon.restore()
  })
  it('should run properly', async function () {
    log('starting stubbed cover-traffic daemon')
    await node.start()
  })
})
