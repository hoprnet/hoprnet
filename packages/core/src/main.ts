import { expandVars, HoprDB, PublicKey } from "@hoprnet/hopr-utils"
import HoprEthereum from "@hoprnet/hopr-core-ethereum"
import PeerId from "peer-id"
import Hopr, { HoprOptions, VERSION } from "."

export function createHoprNode(peerId: PeerId, options: HoprOptions) {
  const db = new HoprDB(
    PublicKey.fromPrivKey(peerId.privKey.marshal()),
    options.createDbIfNotExist,
    VERSION,
    options.dbPath,
    options.forceCreateDB
  )
  const provider = expandVars(options.environment.network.default_provider, process.env)
  const chain = new HoprEthereum(db, PublicKey.fromPeerId(peerId), peerId.privKey.marshal(), {
    chainId: options.environment.network.chain_id,
    environment: options.environment.id,
    gasPrice: options.environment.network.gasPrice,
    network: options.environment.network.id,
    provider
  })
  const node = new Hopr(peerId, db, chain, options)
  return node;
}