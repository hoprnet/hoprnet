import assert from 'assert'
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
  Database
} from '../../../core-ethereum/lib/core_ethereum_db.js'
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
  const pub = PublicKey.from_privkey(stringToU8a('0x1464586aeaea0eb5736884ca1bf42d165fc8e2243b1d917130fb9e321d7a93b8'))
  return new ChannelEntry(
    pub.clone(),
    pub.clone(),
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
      let result = await db_sanity_test(db)
      assert(result)
    } catch (e) {
      assert('EVERYTHING SHOULD PASS', e.toString())
    }
  })
})

describe('db functional tests', function () {
  let db: Database
  let db_shim: LevelDb
  let db_dir_path: string

  beforeEach(async function () {
    db_shim = new LevelDb()
    db_dir_path = '/tmp/test-shim.db'
    await db_shim.init(true, db_dir_path, true, 'monte_rosa')
    db = new Database(db_shim, MOCK_PUBLIC_KEY())
  })

  afterEach(async function () {
    await db_shim.close()

    fs.rmSync(db_dir_path, { recursive: true, force: true })
  })

  it('should store hopr balance', async function () {
    assert((await db.get_hopr_balance()).eq(Balance.zero(BalanceType.HOPR)))

    await db.set_hopr_balance(new Balance('10', BalanceType.HOPR))
    assert.equal((await db.get_hopr_balance()).to_string(), '10')

    await db.add_hopr_balance(new Balance('1', BalanceType.HOPR), TestingSnapshot)
    assert.equal((await db.get_hopr_balance()).to_string(), '11')

    await db.sub_hopr_balance(new Balance('2', BalanceType.HOPR), TestingSnapshot)
    assert.equal((await db.get_hopr_balance()).to_string(), '9')
  })

  it('should test register toggle', async function () {
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
    const hoprNode = MOCK_PUBLIC_KEY()
    const account = MOCK_ADDRESS()

    // should be throw when not added
    assert.rejects(() => db.get_account_from_network_registry(hoprNode), 'should throw when account is not registered')

    // should be set
    await db.add_to_network_registry(hoprNode, account, TestingSnapshot)

    let nodes = await db.find_hopr_node_using_account_in_network_registry(account)
    assert(nodes.len() === 1, 'should have only 1 hoprNode registered')
    assert(
      (await db.find_hopr_node_using_account_in_network_registry(account))[0].eq(hoprNode),
      'should match the registered hoprNode'
    )
    assert((await db.get_account_from_network_registry(hoprNode)).eq(account), 'should match account added')

    // should be removed
    await db.remove_from_network_registry(hoprNode, account, TestingSnapshot)

    // TODO!
    // assert.rejects(
    //     () => db.find_hopr_node_using_account_in_network_registry(account),
    //     'should throw when HoprNode is not linked to an account'
    // )
    assert(
      (await db.find_hopr_node_using_account_in_network_registry(account)).len() === 0,
      'should have 0 hoprNode registered'
    )
    // assert.rejects(() => db.get_account_from_network_registry(hoprNode), 'should throw when account is deregistered')
  })

  it('should test eligible', async function () {
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
    assert.equal(await db.get_rejected_tickets_count(), 0)
    assert((await db.get_rejected_tickets_value()).eq(Balance.zero(BalanceType.HOPR)))

    const amount = new BN(1)

    await db.mark_rejected({
      amount: new Balance(amount.toString(10), BalanceType.HOPR)
    } as Ticket)

    assert.equal(await db.get_rejected_tickets_count(), 1)
    assert((await db.get_rejected_tickets_value()).eq(new Balance(amount.toString(10), BalanceType.HOPR)))
  })

  it('should store ChannelEntry', async function () {
    const channelEntry = channelEntryCreateMock()

    await db.update_channel_and_snapshot(channelEntry.get_id(), channelEntry.clone(), TestingSnapshot)

    assert(!!(await db.get_channel(channelEntry.get_id())), 'did not find channel')
    assert((await db.get_channels()).len() === 1, 'did not find channel')
  })

  it('block number workflow', async function () {
    const initialBlockNumber = await db.get_latest_block_number()

    assert(initialBlockNumber == 0, `initial block number must be set to 0`)

    const blockNumber = new BN(23)
    await db.update_latest_block_number(blockNumber.toNumber())

    const latestBlockNumber = await db.get_latest_block_number()

    assert(blockNumber.eqn(latestBlockNumber), `block number must be updated`)
  })

  it('should store current commitment', async function () {
    const DUMMY_CHANNEL = new Hash(new Uint8Array(Hash.size()).fill(0xff))
    const DUMMY_COMMITMENT = new Hash(new Uint8Array(Hash.size()).fill(0xbb))

    await db.set_current_commitment(DUMMY_CHANNEL, DUMMY_COMMITMENT)

    const fromDb = await db.get_current_commitment(DUMMY_CHANNEL)

    assert(fromDb.eq(DUMMY_COMMITMENT))
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


