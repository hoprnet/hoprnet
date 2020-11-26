import type { HoprOptions } from '.'

import { LevelUp } from 'levelup'
import { blue } from 'chalk'
import { deserializeKeyPair, serializeKeyPair, askForPassword, privKeyToPeerId } from './utils'
import debug from 'debug'

const log = debug('hopr-core:identity')

import { NODE_SEEDS, BOOTSTRAP_SEEDS } from '@hoprnet/hopr-demo-seeds'

import PeerId from 'peer-id'
import Multiaddr from 'multiaddr'

import { KeyPair } from './dbKeys'

/**
 * Assemble the addresses that we are using
 */
function getAddrs(id: PeerId, options: HoprOptions): Multiaddr[] {
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

function getDebugId(options: HoprOptions): string {
  let properId = options.id != null && isFinite(options.id)

  let privKey: string

  if (options.bootstrapNode) {
    if (properId) {
      if (options.id > BOOTSTRAP_SEEDS.length) {
        throw Error(
          `Unable to access bootstrap seed number ${options.id} out of ${BOOTSTRAP_SEEDS.length} bootstrap seeds.`
        )
      }
      privKey = BOOTSTRAP_SEEDS[options.id]
    }
    privKey = BOOTSTRAP_SEEDS[0]
  } else {
    if (properId) {
      if (options.id >= NODE_SEEDS.length) {
        throw Error(`Unable to access node seed number ${options.id} out of ${NODE_SEEDS.length} node seeds.`)
      }
      privKey = NODE_SEEDS[options.id]
    }
  }

  return privKey
}

async function getPeerId(options: HoprOptions): Promise<PeerId> {
  if (options.peerId != null && PeerId.isPeerId(options.peerId)) {
    return options.peerId
  }

  if (options.id != null && isFinite(options.id) && !options.debug) {
    throw Error(`Demo Ids are only available in DEBUG_MODE. Consider setting DEBUG_MODE to 'true' in .env`)
  }

  let privKey: Uint8Array | string

  if (options.debug) {
    privKey = getDebugId(options)

    if (privKey != null) {
      return privKeyToPeerId(privKey)
    }

    log(`Warning: Running in debug mode with keypair stored in database.`)
  }

  if (options.db == null) {
    throw Error('Cannot get/store any peerId without a database handle.')
  }

  return getFromDatabase(options.db, options.password)
}

/**
 * Try to retrieve Id from database
 * @param db database handle
 * @param pw password to keypair decrypt
 */
async function getFromDatabase(db: LevelUp, pw?: string): Promise<PeerId> {
  let serializedKeyPair: Uint8Array

  try {
    serializedKeyPair = await db.get(Buffer.from(KeyPair))
  } catch (err) {
    log('Error loading keys from db', err)
    // No identity in database
    return createIdentity(db, pw)
  }

  return recoverIdentity(serializedKeyPair, pw)
}

async function recoverIdentity(serializedKeyPair: Uint8Array, pw?: string): Promise<PeerId> {
  let peerId: PeerId

  if (pw !== undefined) {
    try {
      return await deserializeKeyPair(serializedKeyPair, new TextEncoder().encode(pw))
    } catch (err) {
      // Exit with error message
      console.log(`Could not recover id from database with given password.`)
      process.exit(1)
    }
  }

  while (true) {
    pw = await askForPassword('Please type in the password that was used to encrypt to key.')

    try {
      peerId = await deserializeKeyPair(serializedKeyPair, new TextEncoder().encode(pw))
      break
    } catch {}
  }

  log(`Successfully recovered ${blue(peerId.toB58String())} from database.`)

  return peerId
}

async function createIdentity(db: LevelUp, pw?: string): Promise<PeerId> {
  pw = pw !== undefined ? pw : await askForPassword('Please type in a password to encrypt the secret key.')

  const peerId = await PeerId.create({ keyType: 'secp256k1' })

  const serializedKeyPair = await serializeKeyPair(peerId, new TextEncoder().encode(pw))

  await db.put(Buffer.from(KeyPair), serializedKeyPair)

  return peerId
}

export default async function getIdentity(options: HoprOptions) {
  let id = await getPeerId(options)

  return {
    id,
    addresses: getAddrs(id, options)
  }
}
