import { debug, expandVars, HoprDB, PublicKey } from "@hoprnet/hopr-utils";
import HoprCoreEthereum from '@hoprnet/hopr-core-ethereum'
import { expect } from "chai";
import PeerId from "peer-id";
import sinon from "sinon";
import Hopr, { HoprOptions, resolveEnvironment, VERSION } from "."

const log = debug('hopr-core:test:index')

describe('hopr controller ', async function () {

  let peerId: PeerId, db: HoprDB, chain: HoprCoreEthereum, options: HoprOptions;
  beforeEach(async function () {
    peerId = await PeerId.create({ keyType: 'secp256k1', bits: 256 })
    options = new HoprOptions(resolveEnvironment('master-xdai'))
    db = new HoprDB(
      PublicKey.fromPrivKey(peerId.privKey.marshal()),
      true,
      VERSION,
      options.dbPath,
      options.forceCreateDB
    )
    chain = new HoprCoreEthereum(db, PublicKey.fromPeerId(peerId), peerId.privKey.marshal(), {
      chainId: options.environment.network.chain_id,
      environment: options.environment.id,
      gasPrice: options.environment.network.gasPrice,
      network: options.environment.network.id,
      provider: expandVars(options.environment.network.default_provider, process.env)
    })
  })

  afterEach(function () {
    sinon.restore()
  })

  it('should start a hopr node properly', async function () {
    log('Starting hopr node...')
    const node = new Hopr(peerId, db, chain, options);
    log('Node started with Id', node.getId().toB58String())
    expect(node instanceof Hopr)
  })
})