import { debug, HoprDB, PublicKey } from "@hoprnet/hopr-utils";
import { expect } from "chai";
import PeerId from "peer-id";
import sinon from "sinon";
import Hopr, { HoprOptions, resolveEnvironment, VERSION } from "."

const log = debug('hopr-core:test:index')

describe('hopr controller ', async function () {

  let peerId: PeerId, db: HoprDB, options: HoprOptions;
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
  })

  afterEach(function () {
    sinon.restore()
  })

  it('should start a hopr node properly', async function () {
    log('Starting hopr node...')
    const node = new Hopr(peerId, db, options);
    log('Node started with Id', node.getId().toB58String())
    expect(node instanceof Hopr)
  })
})