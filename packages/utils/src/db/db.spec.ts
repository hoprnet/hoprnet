import assert from 'assert'
import { randomBytes } from 'crypto'
import {
  Ticket,
  Balance,
  BalanceType,
  Hash,
  U256,
  PublicKey,
  Address,
  Database,
  ChainKeypair,
  EthereumChallenge
} from '../types.js'

import BN from 'bn.js'
import { stringToU8a } from '../u8a/index.js'

export const SECP256K1_CONSTANTS = {
  PRIVATE_KEY_LENGTH: 32,
  COMPRESSED_PUBLIC_KEY_LENGTH: 33,
  UNCOMPRESSED_PUBLIC_KEY_LENGTH: 65,
  SIGNATURE_LENGTH: 64,
  RECOVERABLE_SIGNATURE_LENGTH: 65
}

const MOCK_PUBLIC_KEY = () =>
  PublicKey.deserialize(stringToU8a('0x021464586aeaea0eb5736884ca1bf42d165fc8e2243b1d917130fb9e321d7a93b8'))

const MOCK_ADDRESS = () => Address.from_string('Cf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9')

import { LevelDb } from './db.js'
import { db_sanity_test } from '../../../hoprd/lib//hoprd_hoprd.js'
import fs from 'fs'

function createMockedTicket(signerPrivKey: Uint8Array, counterparty: Address, balance: Balance) {
  let chainKp = new ChainKeypair(signerPrivKey)
  let tkt = new Ticket(
    counterparty,
    balance,
    U256.zero(),
    U256.one(),
    1.0,
    U256.one(),
    new EthereumChallenge(new Uint8Array(20)),
    chainKp,
    new Hash(new Uint8Array(32))
  )
  return tkt
}

describe('db shim tests', function () {
  let db: LevelDb
  let db_dir_path: string

  beforeEach(function () {
    db = new LevelDb()
    db_dir_path = '/tmp/test-shim.db'
  })

  afterEach(async function () {
    await db.close()

    fs.rmSync(db_dir_path, { recursive: true, force: true })
  })

  it('basic DB operations are performed in Rust correctly', async function () {
    await db.init(true, db_dir_path, true, 'monte_rosa')

    try {
      await db_sanity_test(db)
    } catch (e) {
      assert.fail(`db sanity tests should pass: ${e}`)
    }
  })
})

function test_in_memory_db() {
  return new Database(new LevelDb(), MOCK_PUBLIC_KEY().to_address())
}

describe('db functional tests', function () {
  it('should store hopr balance', async function () {
    let db = test_in_memory_db()

    assert((await db.get_hopr_balance()).eq(Balance.zero(BalanceType.HOPR)))

    await db.set_hopr_balance(new Balance('10', BalanceType.HOPR))
    assert.equal((await db.get_hopr_balance()).to_string(), '10')
  })

  it('should store rejected tickets statistics', async function () {
    let db = test_in_memory_db()

    assert.equal(await db.get_rejected_tickets_count(), 0)
    assert((await db.get_rejected_tickets_value()).eq(Balance.zero(BalanceType.HOPR)))

    const amount = new BN(1)

    let ticket = createMockedTicket(
      Uint8Array.from(randomBytes(SECP256K1_CONSTANTS.PRIVATE_KEY_LENGTH)),
      MOCK_ADDRESS(),
      new Balance(amount.toString(10), BalanceType.HOPR)
    )
    await db.mark_rejected(ticket)

    assert.equal(await db.get_rejected_tickets_count(), 1)
    assert((await db.get_rejected_tickets_value()).eq(new Balance(amount.toString(10), BalanceType.HOPR)))
  })

  it('block number workflow', async function () {
    let db = test_in_memory_db()

    const initialBlockNumber = await db.get_latest_block_number()

    assert(initialBlockNumber == 0, `initial block number must be set to 0`)

    const blockNumber = new BN(23)
    await db.update_latest_block_number(blockNumber.toNumber())

    const latestBlockNumber = await db.get_latest_block_number()

    assert(blockNumber.eqn(latestBlockNumber), `block number must be updated`)
  })

  it('should set a packet tag', async function () {
    let db = test_in_memory_db()

    const DUMMY_TAG = new Uint8Array(Hash.size()).fill(0xff)

    await db.check_and_set_packet_tag(DUMMY_TAG)
  })
})

describe(`levelup shim tests`, function () {
  let db: LevelDb

  beforeEach(function () {
    db = new LevelDb()
  })

  afterEach(async function () {
    await db.close()
  })

  it('should store network', async function () {
    await db.setNetworkId('test-env')
    assert.equal(await db.getNetworkId(), 'test-env')
  })

  it('should verify network', async function () {
    await db.setNetworkId('test-env')
    assert((await db.verifyNetworkId('wrong-id')) === false, `must fail for wrong id`)
    assert((await db.verifyNetworkId('test-env')) === true, `must not fail for correct id`)
  })
})
