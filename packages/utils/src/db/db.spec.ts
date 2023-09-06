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

import { rmSync } from 'fs'

import { test_nodejs_env, hoprd_hoprd_initialize_crate } from '../../../hoprd/lib//hoprd_hoprd.js'
hoprd_hoprd_initialize_crate()

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

function test_in_memory_db() {
  return Database.new_in_memory(MOCK_PUBLIC_KEY().to_address())
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

  it('test rusty level db', async function () {
    test_nodejs_env('/tmp')
  })

  it('test db creation and simple set', async function () {
    rmSync('/tmp/test', { force: true, recursive: true })

    let db = new Database('/tmp/test', MOCK_PUBLIC_KEY().to_address())
    let balance_1 = new Balance('100', BalanceType.HOPR)
    await db.set_hopr_balance(balance_1)
    let balance_2 = await db.get_hopr_balance()
    assert.equal(balance_2.to_string(), balance_1.to_string(), 'value must be equal')
  })
})
