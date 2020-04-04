import { keys } from 'libp2p-crypto'
import { LevelUp } from 'levelup'
import chalk from 'chalk'
import { deserializeKeyPair, serializeKeyPair, askForPassword } from '..'

import PeerInfo from 'peer-info'
import PeerId from 'peer-id'

// @ts-ignore
const Multiaddr = require('multiaddr')

import { NAME } from '../../constants'

/**
 * Assemble the addresses that we are using
 */
function getAddrs(options: any): any[] {
  const addrs = []

  if (process.env.PORT == null) {
    throw Error('Unknown port. Please specify a port in the .env file!')
  }

  let port = process.env.PORT

  if (process.env.HOST_IPV4) {
    // ============================= Only for testing ============================================================
    if (options != null && options.id != null && Number.isInteger(options.id)) {
      port = (Number.parseInt(process.env.PORT) + (options.id + 1) * 2).toString()
    }
    // ===========================================================================================================
    addrs.push(Multiaddr(`/ip4/${process.env.HOST_IPV4}/tcp/${port}`))
  }

  // if (process.env.HOST_IPV6) {
  //     // ============================= Only for testing ============================================================
  //     if (Number.isInteger(options.id)) port = (Number.parseInt(process.env.PORT) + (options.id + 1) * 2).toString()
  //     // ===========================================================================================================
  //     addrs.push(Multiaddr(`/ip6/${process.env.HOST_IPV6}/tcp/${Number.parseInt(port) + 1}`))
  // }

  return addrs
}

/**
 * Checks whether we have gotten any peerId in through the options.
 */
async function getPeerId(options: any, db?: LevelUp): Promise<PeerId> {
  if (options != null && options.peerId != null && PeerId.isPeerId(options.peerId)) {
    return options.peerId
  }

  if (db === undefined) {
    throw Error('Cannot get/store any peerId without a database handle.')
  }

  return getFromDatabase(db)
}

/**
 * Try to retrieve Id from database
 */
async function getFromDatabase(db: LevelUp): Promise<PeerId> {
  let peerId: PeerId
  let pw: string
  try {
    const serializedKeyPair = await db.get('key-pair')

    let done = false
    do {
      pw = await askForPassword('Please type in the passwort that was used to encrypt to key.')

      try {
        peerId = await deserializeKeyPair(serializedKeyPair, new TextEncoder().encode(pw))
        done = true
      } catch {}
    } while (!done)

    console.log(`Successfully recovered ${chalk.blue(peerId.toB58String())} from database.`)
  } catch (err) {
    if (err != null && err.notFound != true) {
      throw err
    }

    pw = await askForPassword('Please type in a password to encrypt the secret key.')

    const key = await keys.generateKeyPair('secp256k1', 256)
    peerId = await PeerId.createFromPrivKey(key.bytes)

    const serializedKeyPair = await serializeKeyPair(peerId, new TextEncoder().encode(pw))
    await db.put('key-pair', serializedKeyPair)
  }

  return peerId
}

async function getPeerInfo(
  options: {
    id?: number
    bootstrapNode?: boolean
    peerId?: PeerId
    peerInfo?: PeerInfo
    addrs?: any[]
  },
  db?: LevelUp
): Promise<PeerInfo> {
  if (db == null && (options == null || (options != null && options.peerInfo == null && options.peerId == null))) {
    throw Error('Invalid input parameter. Please set a valid peerInfo or pass a database handle.')
  }

  /**
   * Check whether our config makes sense
   */
  function checkConfig(): void {
    if (!process.env.HOST_IPV4 && !process.env.HOST_IPV6) {
      throw Error('Unable to start node without an address. Please provide at least one.')
    }

    if (!process.env.PORT) {
      throw Error('Got no port to listen on. Please specify one.')
    }
  }

  checkConfig()

  options.addrs = getAddrs(options)

  let peerInfo: PeerInfo
  if (options.peerInfo != null) {
    peerInfo = options.peerInfo
  } else {
    peerInfo = new PeerInfo(await getPeerId(options, db))
  }

  options.addrs.forEach(addr => peerInfo.multiaddrs.add(addr.encapsulate(`/${NAME}/${peerInfo.id.toB58String()}`)))

  return peerInfo
}

export { getPeerInfo }
