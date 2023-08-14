import assert from 'assert'
import { randomBytes } from 'crypto'
import {
  Ticket,
  Balance,
  BalanceType,
  Hash,
  U256,
  ChannelEntry,
  PublicKey,
  Address,
  Snapshot,
  ChannelStatus,
  Database,
  Response,
  ChainKeypair,
  core_ethereum_db_initialize_crate
} from '../../../core-ethereum/lib/core_ethereum_db.js'
core_ethereum_db_initialize_crate()

import BN from 'bn.js'
import { stringToU8a } from '../u8a/index.js'

export const SECP256K1_CONSTANTS = {
  PRIVATE_KEY_LENGTH: 32,
  COMPRESSED_PUBLIC_KEY_LENGTH: 33,
  UNCOMPRESSED_PUBLIC_KEY_LENGTH: 65,
  SIGNATURE_LENGTH: 64,
  RECOVERABLE_SIGNATURE_LENGTH: 65
}

const TestingSnapshot = new Snapshot(U256.zero(), U256.zero(), U256.zero())

const MOCK_PUBLIC_KEY = () =>
  PublicKey.deserialize(stringToU8a('0x021464586aeaea0eb5736884ca1bf42d165fc8e2243b1d917130fb9e321d7a93b8'))

const MOCK_ADDRESS = () => Address.from_string('Cf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9')

function channelEntryCreateMock(): ChannelEntry {
  const src = Address.from_string('0x86d854baec85640ef8c80b2c618c28024f2926d4')
  const dest = Address.from_string('0x245445dbdcdaa115bb1d7d1de8717f9facecdbbe')
  return new ChannelEntry(
    src,
    dest,
    new Balance('1', BalanceType.HOPR),
    Hash.create([]),
    U256.one(),
    U256.one(),
    ChannelStatus.Closed,
    U256.one(),
    U256.one()
  )
}

import { LevelDb } from './db.js'
import { db_sanity_test } from '../../lib/utils_db.js'
import fs from 'fs'

function createMockedTicket(signerPrivKey: Uint8Array, counterparty: Address, balance: Balance) {
  let chainKp = new ChainKeypair(signerPrivKey)
  let tkt = Ticket.new(
    counterparty,
    U256.zero(),
    U256.zero(),
    balance,
    U256.from_inverse_probability(U256.one()),
    U256.one(),
    chainKp
  )
  tkt.set_challenge(new Response(Uint8Array.from(randomBytes(32))).to_challenge().to_ethereum_challenge(), chainKp)
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

    await db.add_hopr_balance(new Balance('1', BalanceType.HOPR), TestingSnapshot)
    assert.equal((await db.get_hopr_balance()).to_string(), '11')

    await db.sub_hopr_balance(new Balance('2', BalanceType.HOPR), TestingSnapshot)
    assert.equal((await db.get_hopr_balance()).to_string(), '9')
  })

  it('should test register toggle', async function () {
    let db = test_in_memory_db()

    // should be false by default
    assert((await db.is_network_registry_enabled()) === true, 'register should be enabled by default')

    // should be true once set
    await db.set_network_registry(true, TestingSnapshot)
    assert((await db.is_network_registry_enabled()) === true, 'register should be enabled')

    // should be false once unset
    await db.set_network_registry(false, TestingSnapshot)
    assert((await db.is_network_registry_enabled()) === false, 'register should be disabled')
  })

  it('should test registry', async function () {
    let db = test_in_memory_db()

    const hoprNode = MOCK_PUBLIC_KEY()
    const account = MOCK_ADDRESS()

    // should be throw when not added
    assert.equal(await db.get_account_from_network_registry(hoprNode.to_address()), undefined)

    // should be set
    await db.add_to_network_registry(hoprNode.to_address(), account, TestingSnapshot)

    let nodes = await db.find_hopr_node_using_account_in_network_registry(account)
    assert(nodes.len() === 1, 'should have only 1 hoprNode registered')
    assert(
      (await db.find_hopr_node_using_account_in_network_registry(account)).next().eq(hoprNode.to_address()),
      'should match the registered hoprNode'
    )
    assert(
      (await db.get_account_from_network_registry(hoprNode.to_address())).eq(account),
      'should match account added'
    )

    // should be removed
    await db.remove_from_network_registry(hoprNode.to_address(), account, TestingSnapshot)

    assert(
      (await db.find_hopr_node_using_account_in_network_registry(account)).len() === 0,
      'should have 0 hoprNode registered'
    )
    assert.equal(await db.get_account_from_network_registry(hoprNode.to_address()), undefined)
  })

  it('should test eligible', async function () {
    let db = test_in_memory_db()

    const account = MOCK_ADDRESS()

    // should be false by default
    assert((await db.is_eligible(account)) === false, 'account is not eligible by default')

    // should be true once set
    await db.set_eligible(account, true, TestingSnapshot.clone())
    assert((await db.is_eligible(account)) === true, 'account should be eligible')

    // should be false once unset
    await db.set_eligible(account, false, TestingSnapshot.clone())
    assert((await db.is_eligible(account)) === false, 'account should not be eligible')
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

  it('should store ChannelEntry', async function () {
    let db = test_in_memory_db()

    const channelEntry = channelEntryCreateMock()

    await db.update_channel_and_snapshot(channelEntry.get_id(), channelEntry.clone(), TestingSnapshot)

    assert((await db.get_channel(channelEntry.get_id())) !== undefined)
    assert.equal((await db.get_channels()).len(), 1, 'did not find channel')
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

  it('should store current commitment', async function () {
    let db = test_in_memory_db()

    const DUMMY_CHANNEL = new Hash(new Uint8Array(Hash.size()).fill(0xff))
    const DUMMY_COMMITMENT = new Hash(new Uint8Array(Hash.size()).fill(0xbb))

    await db.set_current_commitment(DUMMY_CHANNEL, DUMMY_COMMITMENT)

    const fromDb = await db.get_current_commitment(DUMMY_CHANNEL)

    assert(fromDb.eq(DUMMY_COMMITMENT))
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
