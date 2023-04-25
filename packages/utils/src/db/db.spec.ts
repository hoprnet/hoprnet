import { HoprDB } from './db.js'
import { randomBytes } from 'crypto'

import assert from 'assert'
import {
  UnacknowledgedTicket,
  Ticket,
  AcknowledgedTicket,
  Balance,
  Hash,
  UINT256,
  HalfKey,
  Response,
  HalfKeyChallenge,
  ChannelEntry,
  PublicKey,
  Address,
  Snapshot
} from '../types/index.js'
import BN from 'bn.js'
import { SECP256K1_CONSTANTS } from '../crypto/index.js'
import { u8aEquals } from '../u8a/index.js'

const TestingSnapshot = new Snapshot(new BN(0), new BN(0), new BN(0))

class TestingDB extends HoprDB {
  public async get(key: Uint8Array) {
    return await super.get(key)
  }

  public async put(key: Uint8Array, value: Uint8Array) {
    return await super.put(key, value)
  }

  public async getAll<T, U = T>(
    range: {
      prefix: Uint8Array
      suffixLength: number
    },
    deserialize: (chunk: Uint8Array) => T,
    filter?: (o: T) => boolean,
    map?: (i: T) => U,
    sort?: (e1: U, e2: U) => number
  ) {
    return await super.getAll<T, U>(range, deserialize, filter, map, sort)
  }

  public static createMock(id?: PublicKey) {
    return super.createMock(id) as TestingDB
  }
}

function createMockedTicket(signerPrivKey: Uint8Array, counterparty: Address) {
  return Ticket.create(
    counterparty,
    new Response(Uint8Array.from(randomBytes(32))).toChallenge(),
    UINT256.fromString('0'),
    UINT256.fromString('0'),
    new Balance(new BN(0)),
    UINT256.fromInverseProbability(new BN(1)),
    UINT256.fromString('1'),
    signerPrivKey
  )
}

describe(`database tests`, function () {
  let db: TestingDB

  beforeEach(function () {
    db = TestingDB.createMock()
  })

  afterEach(async function () {
    await db.close()
  })

  it('getAll - basic', async function () {
    const TEST_KEY = new TextEncoder().encode(`test key`)
    const TEST_VALUE = new TextEncoder().encode(`test value`)
    await db.put(TEST_KEY, TEST_VALUE)

    const resultSingle = await db.getAll({ prefix: TEST_KEY, suffixLength: 0 }, (x) => x)

    assert(resultSingle.length == 1)
    assert(u8aEquals(TEST_VALUE, resultSingle[0]))

    const TEST_KEY_RANGE = Uint8Array.from([...TEST_KEY, 23])

    await db.put(TEST_KEY_RANGE, TEST_VALUE)

    const resultRange = await db.getAll({ prefix: TEST_KEY, suffixLength: 1 }, (x) => x)

    assert(resultRange.length == 1)
    assert(u8aEquals(TEST_VALUE, resultRange[0]))

    assert((await db.getAll({ prefix: TEST_KEY, suffixLength: 0 }, (x) => x)).length == 1)
  })

  it('getAll - filter', async function () {
    const TEST_KEY = new TextEncoder().encode(`test key`)
    const TEST_VALUE = Uint8Array.from([...new TextEncoder().encode(`test value`), 23])

    await db.put(TEST_KEY, TEST_VALUE)

    const resultTrue = await db.getAll(
      { prefix: TEST_KEY, suffixLength: 0 },
      (x) => x,
      (value: Uint8Array) => value[value.length - 1] == 23
    )

    assert(resultTrue.length == 1)
    assert(u8aEquals(TEST_VALUE, resultTrue[0]))

    const resultFalse = await db.getAll(
      { prefix: TEST_KEY, suffixLength: 0 },
      (x) => x,
      (value: Uint8Array) => value[value.length - 1] == 24
    )

    assert(resultFalse.length == 0)
  })

  it('getAll - map', async function () {
    const TEST_KEY = new TextEncoder().encode(`test key`)
    const TEST_VALUE = Uint8Array.from([...new TextEncoder().encode(`test value`), 23])
    const MAPPED_VALUE = new TextEncoder().encode(`mapped value`)

    await db.put(TEST_KEY, TEST_VALUE)

    const resultMapped = await db.getAll(
      { prefix: TEST_KEY, suffixLength: 0 },
      (x) => x,
      undefined,
      (input: Uint8Array) => {
        if (u8aEquals(input, TEST_VALUE)) {
          return MAPPED_VALUE
        } else {
          return input
        }
      }
    )

    assert(resultMapped.length == 1)
    assert(u8aEquals(MAPPED_VALUE, resultMapped[0]))
  })

  it('getAll - sort', async function () {
    const TEST_KEY = new TextEncoder().encode(`test key`)
    const TEST_VALUE_FIRST = Uint8Array.from([...new TextEncoder().encode(`test value`), 0])
    const TEST_VALUE_MIDDLE = Uint8Array.from([...new TextEncoder().encode(`test value`), 1])
    const TEST_VALUE_LAST = Uint8Array.from([...new TextEncoder().encode(`test value`), 2])

    await db.put(Uint8Array.from([...TEST_KEY, 0]), TEST_VALUE_LAST)
    await db.put(Uint8Array.from([...TEST_KEY, 1]), TEST_VALUE_FIRST)
    await db.put(Uint8Array.from([...TEST_KEY, 2]), TEST_VALUE_MIDDLE)

    const resultSorted = await db.getAll(
      { prefix: TEST_KEY, suffixLength: 1 },
      (x) => x,
      undefined,
      undefined,
      (a: Uint8Array, b: Uint8Array) => a[a.length - 1] - b[b.length - 1]
    )

    assert(resultSorted.every((value: Uint8Array, index: number) => value[value.length - 1] == index))
  })

  it('hasPacket', async function () {
    const packetTag = Uint8Array.from(randomBytes(16))

    const present = await db.checkAndSetPacketTag(packetTag)

    assert(present == false, `Packet tag must not be present`)

    const secondTry = await db.checkAndSetPacketTag(packetTag)

    assert(secondTry == true, `Packet tag must be set`)
  })

  it('ticket workflow', async function () {
    const privKey = randomBytes(SECP256K1_CONSTANTS.PRIVATE_KEY_LENGTH)
    const pubKey = PublicKey.fromPrivKey(privKey)
    // this comes from a Packet
    const halfKeyChallenge = new HalfKeyChallenge(Uint8Array.from(randomBytes(HalfKeyChallenge.SIZE)))
    const unAck = new UnacknowledgedTicket(
      createMockedTicket(privKey, new Address(randomBytes(Address.SIZE))),
      new HalfKey(Uint8Array.from(randomBytes(HalfKey.SIZE))),
      pubKey
    )
    await db.storePendingAcknowledgement(halfKeyChallenge, false, unAck)
    assert((await db.getTickets()).length == 1, `DB should find one ticket`)

    const pending = await db.getPendingAcknowledgement(halfKeyChallenge)

    assert(pending.isMessageSender == false)

    const ack = new AcknowledgedTicket(
      pending.ticket.ticket,
      new Response(Uint8Array.from(randomBytes(Hash.SIZE))),
      new Hash(Uint8Array.from(randomBytes(Hash.SIZE))),
      pubKey
    )
    await db.replaceUnAckWithAck(halfKeyChallenge, ack)

    assert((await db.getTickets()).length == 1, `DB should find one ticket`)
    assert((await db.getUnacknowledgedTickets()).length === 0, `DB should not contain any unacknowledgedTicket`)
    assert((await db.getAcknowledgedTickets()).length == 1, `DB should contain exactly one acknowledged ticket`)
  })

  it('block number workflow', async function () {
    const initialBlockNumber = await db.getLatestBlockNumber()

    assert(initialBlockNumber == 0, `initial block number must be set to 0`)

    const blockNumber = new BN(23)
    await db.updateLatestBlockNumber(blockNumber)

    const latestBlockNumber = await db.getLatestBlockNumber()

    assert(blockNumber.eqn(latestBlockNumber), `block number must be updated`)
  })

  it('should store ChannelEntry', async function () {
    const channelEntry = ChannelEntry.createMock()

    await db.updateChannelAndSnapshot(channelEntry.getId(), channelEntry, TestingSnapshot)

    assert(!!(await db.getChannel(channelEntry.getId())), 'did not find channel')
    assert((await db.getChannels()).length === 1, 'did not find channel')
  })

  it('should store ticketIndex', async function () {
    const DUMMY_CHANNEL = new Hash(new Uint8Array(Hash.SIZE).fill(0xff))
    const DUMMY_INDEX = new UINT256(new BN(1))

    await db.setCurrentTicketIndex(DUMMY_CHANNEL, DUMMY_INDEX)

    const fromDb = await db.getCurrentTicketIndex(DUMMY_CHANNEL)

    assert(fromDb.toBN().eq(DUMMY_INDEX.toBN()))
  })

  it('should store current commitment', async function () {
    const DUMMY_CHANNEL = new Hash(new Uint8Array(Hash.SIZE).fill(0xff))
    const DUMMY_COMMITMENT = new Hash(new Uint8Array(Hash.SIZE).fill(0xbb))

    await db.setCurrentCommitment(DUMMY_CHANNEL, DUMMY_COMMITMENT)

    const fromDb = await db.getCurrentCommitment(DUMMY_CHANNEL)

    assert(fromDb.eq(DUMMY_COMMITMENT))
  })

  it('should store rejected tickets statistics', async function () {
    assert.equal(await db.getRejectedTicketsCount(), 0)
    assert((await db.getRejectedTicketsValue()).toBN().isZero())

    const amount = new BN(1)

    await db.markRejected({
      amount: new Balance(amount)
    } as Ticket)

    assert.equal(await db.getRejectedTicketsCount(), 1)
    assert((await db.getRejectedTicketsValue()).toBN().eq(amount))
  })

  it('should store environment', async function () {
    await db.setEnvironmentId('test-env')
    assert.equal(await db.getEnvironmentId(), 'test-env')
  })

  it('should verify environment', async function () {
    await db.setEnvironmentId('test-env')
    assert((await db.verifyEnvironmentId('wrong-id')) === false, `must fail for wrong id`)
    assert((await db.verifyEnvironmentId('test-env')) === true, `must not fail for correct id`)
  })

  it('should store hopr balance', async function () {
    assert((await db.getHoprBalance()).toBN().isZero())

    await db.setHoprBalance(new Balance(new BN(10)))
    assert.equal((await db.getHoprBalance()).toString(), '10')

    await db.addHoprBalance(new Balance(new BN(1)), TestingSnapshot)
    assert.equal((await db.getHoprBalance()).toString(), '11')

    await db.subHoprBalance(new Balance(new BN(2)), TestingSnapshot)
    assert.equal((await db.getHoprBalance()).toString(), '9')
  })

  it('should test registry', async function () {
    const hoprNode = PublicKey.createMock()
    const account = Address.createMock()

    // should be throw when not added
    assert.rejects(() => db.getAccountFromNetworkRegistry(hoprNode), 'should throw when account is not registered')

    // should be set
    await db.addToNetworkRegistry(hoprNode, account, TestingSnapshot)
    assert(
      (await db.findHoprNodesUsingAccountInNetworkRegistry(account)).length === 1,
      'should have only 1 hoprNode registered'
    )
    assert(
      (await db.findHoprNodesUsingAccountInNetworkRegistry(account))[0].eq(hoprNode),
      'should match the registered hoprNode'
    )
    assert((await db.getAccountFromNetworkRegistry(hoprNode)).eq(account), 'should match account added')

    // should be removed
    await db.removeFromNetworkRegistry(hoprNode, account, TestingSnapshot)
    assert.rejects(
      () => db.findHoprNodesUsingAccountInNetworkRegistry(account),
      'should throw when HoprNode is not linked to an account'
    )
    assert(
      (await db.findHoprNodesUsingAccountInNetworkRegistry(account)).length === 0,
      'should have 0 hoprNode registered'
    )
    assert.rejects(() => db.getAccountFromNetworkRegistry(hoprNode), 'should throw when account is deregistered')
  })

  it('should test eligible', async function () {
    const account = Address.createMock()

    // should be false by default
    assert((await db.isEligible(account)) === false, 'account is not eligible by default')

    // should be true once set
    await db.setEligible(account, true, TestingSnapshot)
    assert((await db.isEligible(account)) === true, 'account should be eligible')

    // should be false once unset
    await db.setEligible(account, false, TestingSnapshot)
    assert((await db.isEligible(account)) === false, 'account should be uneligible')
  })

  it('should test register toggle', async function () {
    // should be false by default
    assert((await db.isNetworkRegistryEnabled()) === true, 'register should be enabled by default')

    // should be true once set
    await db.setNetworkRegistryEnabled(true, TestingSnapshot)
    assert((await db.isNetworkRegistryEnabled()) === true, 'register should be enabled')

    // should be false once unset
    await db.setNetworkRegistryEnabled(false, TestingSnapshot)
    assert((await db.isNetworkRegistryEnabled()) === false, 'register should be disabled')
  })

  it('putSerializedObject and getSerializedObject should store and read object', async function () {
    const ns = 'testobjects'
    const key = '1'
    const object = Uint8Array.from(randomBytes(32))

    await db.putSerializedObject(ns, key, object)
    const storedObject = await db.getSerializedObject(ns, key)
    assert(storedObject !== undefined, 'storedObject should not be undefined')
    assert(u8aEquals(object, storedObject), 'storedObject should equal object')
  })

  it('putSerializedObject should update object', async function () {
    const ns = 'testobjects'
    const key = '2'
    const object1 = Uint8Array.from(randomBytes(32))
    const object2 = Uint8Array.from(randomBytes(32))

    await db.putSerializedObject(ns, key, object1)
    await db.putSerializedObject(ns, key, object2)

    const storedObject = await db.getSerializedObject(ns, key)
    assert(storedObject !== undefined, 'storedObject should not be undefined')
    assert(u8aEquals(object2, storedObject), 'storedObject should equal object2')
  })

  it('deleteSerializedObject should delete object', async function () {
    const ns = 'testobjects'
    const key = '3'
    const object = Uint8Array.from(randomBytes(32))

    await db.putSerializedObject(ns, key, object)
    await db.deleteObject(ns, key)

    const storedObject = await db.getSerializedObject(ns, key)
    assert(storedObject === undefined, 'storedObject should be undefined')
  })
})

import { LevelDb } from './db.js'
import {db_sanity_test} from "../../lib/utils_db.js";

describe('db shim tests', function() {
  it('basic DB operations are performed in Rust correctly', async function () {
    let db = new LevelDb()
    await db.init(true, '/tmp/test-shim.db', true, 'monte_rosa')

    try {
      let result = await db_sanity_test(db)
      assert(result)
    } catch (e) {
      assert("", e.toString())
    }
  })
})