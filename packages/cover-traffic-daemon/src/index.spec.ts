import Hopr, { resolveEnvironment } from '@hoprnet/hopr-core'
import { debug, privKeyToPeerId } from '@hoprnet/hopr-utils'
import sinon from 'sinon'
import { generateNode, generateNodeOptions } from '.'

const log = debug('hopr:test:cover-traffic')

describe('cover-traffic daemon', async function () {
  const privateKey = '0xcb1e5d91d46eb54a477a7eefec9c87a1575e3e5384d38f990f19c09aa8ddd332'
  const options = await generateNodeOptions(resolveEnvironment('master-xdai'))

  let node: Hopr;
  beforeEach(function () {
    node = generateNode(privKeyToPeerId(privateKey), options)
    sinon.stub(node.indexer, 'start').callsFake(() => Promise.resolve(log("indexer called for cover-traffic daemon")))
  })
  afterEach(function () {
    sinon.restore()
  })
  it('should run properly', async function () {
    log("starting stubbed cover-traffic daemon")
    await node.start()
  })
})