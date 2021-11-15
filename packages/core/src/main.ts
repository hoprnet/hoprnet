import { expandVars, HoprDB, PublicKey } from '@hoprnet/hopr-utils'
import HoprEthereum from '@hoprnet/hopr-core-ethereum'
import PeerId from 'peer-id'
import { debug } from '@hoprnet/hopr-utils'
import Hopr, { HoprOptions, VERSION } from '.'

const log = debug(`hopr-core:create-hopr`)

/*
 * General function to create a HOPR node given an identity an
 * range of options.
 * @param peerId:PeerId - Identity used by the HOPR node
 * @param options:HoprOptions - Required options to create node
 * @returns {Hopr} - HOPR node
 */
export function createHoprNode(peerId: PeerId, options: HoprOptions): Hopr {
  const db = new HoprDB(
    PublicKey.fromPrivKey(peerId.privKey.marshal()),
    options.createDbIfNotExist,
    VERSION,
    options.dbPath,
    options.forceCreateDB
  )
  const provider = expandVars(options.environment.network.default_provider, process.env)
  log(`using provider URL: ${provider}`)
  const chain = new HoprEthereum(db, PublicKey.fromPeerId(peerId), peerId.privKey.marshal(), {
    chainId: options.environment.network.chain_id,
    environment: options.environment.id,
    gasPrice: options.environment.network.gasPrice,
    network: options.environment.network.id,
    provider
  })
  const node = new Hopr(peerId, db, chain, options)
  return node
}
