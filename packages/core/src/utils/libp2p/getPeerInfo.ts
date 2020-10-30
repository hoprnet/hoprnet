import {HoprOptions} from '../../'

import {keys} from 'libp2p-crypto'
import {LevelUp} from 'levelup'
import chalk from 'chalk'
import {deserializeKeyPair, serializeKeyPair, askForPassword, privKeyToPeerId} from '..'
import debug from 'debug'
const log = debug('hopr-core:libp2p')

import {NODE_SEEDS, BOOTSTRAP_SEEDS} from '@hoprnet/hopr-demo-seeds'

import PeerId from 'peer-id'
import Multiaddr from 'multiaddr'

import {KeyPair} from '../../dbKeys'

/**
 * Assemble the addresses that we are using
 */
async function getAddrs(id: PeerId, options: HoprOptions): Promise<Multiaddr[]> {
  const addrs = []

  if (options.hosts === undefined || (options.hosts.ip4 === undefined && options.hosts.ip6 === undefined)) {
    let ip4Port = 9091
    // ============================= Only for testing ============================================================
    if (options.id != null && Number.isInteger(options.id)) {
      ip4Port += (options.id + 1) * 2
    }
    // ===========================================================================================================
    addrs.push(Multiaddr(`/ip4/0.0.0.0/tcp/${ip4Port}`))
  }

  if (options.hosts !== undefined) {
    if (options.hosts.ip4 === undefined && options.hosts.ip6 === undefined) {
      throw Error(`Unable to detect to which interface we should listen`)
    }
    if (options.hosts.ip4 !== undefined) {
      let ip4Port = options.hosts.ip4.port
      // ============================= Only for testing ============================================================
      if (options.id != null && Number.isInteger(options.id)) {
        ip4Port += (options.id + 1) * 2
      }
      // ===========================================================================================================
      addrs.push(Multiaddr(`/ip4/${options.hosts.ip4.ip}/tcp/${ip4Port}`))
    }

    if (options.hosts.ip6 !== undefined) {
      let ip6Port = options.hosts.ip6.port
      // ============================= Only for testing ============================================================
      if (options.id != null && Number.isInteger(options.id)) {
        ip6Port += (options.id + 1) * 2
      }
      // ===========================================================================================================
      addrs.push(Multiaddr(`/ip6/${options.hosts.ip6.ip}/tcp/${ip6Port}`))
    }
  }
  return addrs.map((addr: Multiaddr) => addr.encapsulate(`/p2p/${id.toB58String()}`))
}

/**
 * Checks whether we have gotten any peerId in through the options.
 */
async function getPeerId(options: HoprOptions, db?: LevelUp): Promise<PeerId> {
  if (options.peerId != null && PeerId.isPeerId(options.peerId)) {
    return options.peerId
  }

  if (options.debug) {
    if (options.id != null && isFinite(options.id)) {
      if (options.bootstrapNode) {
        if (options.id >= BOOTSTRAP_SEEDS.length) {
          throw Error(
            `Unable to access bootstrap seed number ${options.id} out of ${BOOTSTRAP_SEEDS.length} bootstrap seeds.`
          )
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
  } else if (options.id != null && isFinite(options.id)) {
    throw Error(`Demo Ids are only available in DEBUG_MODE. Consider setting DEBUG_MODE to 'true' in .env`)
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
  let serializedKeyPair: Uint8Array
  try {
    serializedKeyPair = (await db.get(Buffer.from(KeyPair))) as Uint8Array
  } catch (err) {
    return createIdentity(db, pw)
  }

  return recoverIdentity(serializedKeyPair, pw)
}

async function recoverIdentity(serializedKeyPair: Uint8Array, pw?: string): Promise<PeerId> {
  let peerId: PeerId | undefined
  let done = false

  if (pw !== undefined) {
    try {
      return await deserializeKeyPair(serializedKeyPair, new TextEncoder().encode(pw))
    } catch (err) {
      log(`Could not recover id from database with given password.`)
      process.exit(0)
    }
  }

  while (!done) {
    pw = await askForPassword('Please type in the password that was used to encrypt to key.')

    try {
      peerId = await deserializeKeyPair(serializedKeyPair, new TextEncoder().encode(pw))
      done = true
    } catch {}
  }

  log(`Successfully recovered ${chalk.blue((peerId as PeerId).toB58String())} from database.`)

  return peerId as PeerId
}

async function createIdentity(db: LevelUp, pw?: string): Promise<PeerId> {
  pw = pw !== undefined ? pw : await askForPassword('Please type in a password to encrypt the secret key.')

  const key = await keys.generateKeyPair('secp256k1', 256)
  const peerId = await PeerId.createFromPrivKey(key.bytes)

  const serializedKeyPair = await serializeKeyPair(peerId, new TextEncoder().encode(pw))
  await db.put(Buffer.from(KeyPair), serializedKeyPair)

  return peerId
}

export {getPeerId, getAddrs}
