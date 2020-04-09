import { HoprOptions } from '../../'

import { keys } from 'libp2p-crypto'
import { LevelUp } from 'levelup'
import chalk from 'chalk'
import { deserializeKeyPair, serializeKeyPair, askForPassword, privKeyToPeerId } from '..'

import { NODE_SEEDS, BOOTSTRAP_SEEDS } from '@hoprnet/hopr-demo-seeds'

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
async function getPeerId(options: HoprOptions, db?: LevelUp): Promise<PeerId> {
  if (options.peerId != null && PeerId.isPeerId(options.peerId)) {
    return options.peerId
  }

  if (process.env.DEVELOP_MODE === 'true') {
    if (options.id != null && isFinite(options.id)) {
      if (options.bootstrapNode) {
        if (options.id >= BOOTSTRAP_SEEDS.length) {
          throw Error(`Unable to access bootstrap seed number ${options.id} out of ${BOOTSTRAP_SEEDS.length} bootstrap seeds.`)
        }
        return await privKeyToPeerId(BOOTSTRAP_SEEDS[options.id])
      } else {
        if (options.id >= NODE_SEEDS.length) {
          throw Error(`Unable to access node seed number ${options.id} out of ${NODE_SEEDS.length} node seeds.`)
        }
        return await privKeyToPeerId(NODE_SEEDS[options.id])
      }
    } else if (options.bootstrapNode) {
      return await privKeyToPeerId(BOOTSTRAP_SEEDS[0])
    }
  }

  if (db == null) {
    throw Error('Cannot get/store any peerId without a database handle.')
  }

  return getFromDatabase(db, options.password)
}

/**
 * Try to retrieve Id from database
 */
async function getFromDatabase(db: LevelUp, pw?: string): Promise<PeerId> {
  let peerId: PeerId
  try {
    const serializedKeyPair = await db.get('key-pair')

    let done = false
    do {
      pw = pw || (await askForPassword('Please type in the passwort that was used to encrypt to key.'))

      try {
        peerId = await deserializeKeyPair(serializedKeyPair, new TextEncoder().encode(pw))
        done = true
      } catch {
        pw = undefined
      }
    } while (!done)

    console.log(`Successfully recovered ${chalk.blue(peerId.toB58String())} from database.`)
  } catch (err) {
    if (err != null && err.notFound != true) {
      throw err
    }

    pw = pw || (await askForPassword('Please type in a password to encrypt the secret key.'))

    const key = await keys.generateKeyPair('secp256k1', 256)
    peerId = await PeerId.createFromPrivKey(key.bytes)

    const serializedKeyPair = await serializeKeyPair(peerId, new TextEncoder().encode(pw))
    await db.put('key-pair', serializedKeyPair)
  }

  return peerId
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

async function getPeerInfo(options: HoprOptions, db?: LevelUp): Promise<PeerInfo> {
  if (db == null && (options == null || (options != null && options.peerInfo == null && options.peerId == null))) {
    throw Error('Invalid input parameter. Please set a valid peerInfo or pass a database handle.')
  }

  checkConfig()

  const addrs = getAddrs(options)

  let peerInfo: PeerInfo
  if (options.peerInfo != null) {
    peerInfo = options.peerInfo
  } else {
    peerInfo = new PeerInfo(await getPeerId(options, db))
  }

  addrs.forEach(addr => peerInfo.multiaddrs.add(addr.encapsulate(`/${NAME}/${peerInfo.id.toB58String()}`)))

  return peerInfo
}

export { getPeerInfo }
