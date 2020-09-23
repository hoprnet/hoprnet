import Hopr from '../..'
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'

import HoprEthereum from '@hoprnet/hopr-core-ethereum'

import { Ganache } from '@hoprnet/hopr-testing'
import { migrate, fund } from '@hoprnet/hopr-ethereum'

import assert from 'assert'

import { u8aEquals, durations } from '@hoprnet/hopr-utils'

import { MAX_HOPS } from '../../constants'

import LevelUp from 'levelup'
import MemDown from 'memdown'

import Debug from 'debug'
import { ACKNOWLEDGED_TICKET_INDEX_LENGTH } from '../../dbKeys'
const log = Debug(`hopr-core:testing`)

const TWO_SECONDS = durations.seconds(2)

async function startTestnet() {
  const ganache = new Ganache()

  await ganache.start()
  await migrate()
  await fund(4)

  return ganache
}

async function generateNode(id: number): Promise<Hopr<HoprEthereum>> {
  // Start HOPR in DEBUG_MODE and use demo seeds
  const node = (await Hopr.create({
    id,
    db: new LevelUp(MemDown()),
    connector: HoprEthereum,
    provider: GANACHE_URI,
    network: 'ethereum',
    debug: true,
  })) as Hopr<HoprEthereum>

  await node.paymentChannels.initOnchainValues()

  return node
}

const GANACHE_URI = `ws://127.0.0.1:9545`

describe('test packet composition and decomposition', function () {
  let testnet: Ganache

  beforeEach(async function () {
    testnet = await startTestnet()
  }, durations.seconds(30))

  afterEach(async function () {
    await testnet.stop()
  })

  it(
    'should create packets and decompose them',
    async function () {
      const nodes = await Promise.all(Array.from({ length: MAX_HOPS + 1 }).map((_value, index) => generateNode(index)))

      connectionHelper(nodes)

      const testMessages: Uint8Array[] = []

      for (let i = 0; i < MAX_HOPS; i++) {
        testMessages.push(new TextEncoder().encode(`test message #${i}`))
      }

      let msgReceivedPromises = []

      for (let i = 1; i <= MAX_HOPS; i++) {
        msgReceivedPromises.push(receiveChecker(testMessages.slice(i - 1, i), nodes[i]))
        await nodes[0].sendMessage(testMessages[i - 1], nodes[i].peerInfo.id, async () =>
          nodes.slice(1, i).map((node) => node.peerInfo.id)
        )
      }

      const timeout = setTimeout(() => {
        assert.fail(`No message received`)
      }, TWO_SECONDS)

      await Promise.all(msgReceivedPromises)

      clearTimeout(timeout)

      cleanUpReceiveChecker(nodes)

      msgReceivedPromises = []

      msgReceivedPromises.push(receiveChecker(testMessages.slice(1, 3), nodes[nodes.length - 1]))

      for (let i = 1; i <= MAX_HOPS - 1; i++) {
        await nodes[i].sendMessage(testMessages[i], nodes[nodes.length - 1].peerInfo.id, async () =>
          nodes.slice(i + 1, nodes.length - 1).map((node) => node.peerInfo.id)
        )
      }

      await new Promise((resolve) => setTimeout(resolve, 700))

      for (let i = 0; i < nodes.length; i++) {
        const tickets = []

        await new Promise((resolve) =>
          nodes[i].db
            .createValueStream({
              gte: Buffer.from(
                nodes[i].dbKeys.AcknowledgedTickets(Buffer.alloc(ACKNOWLEDGED_TICKET_INDEX_LENGTH, 0x00))
              ),
              lt: Buffer.from(
                nodes[i].dbKeys.AcknowledgedTickets(Buffer.alloc(ACKNOWLEDGED_TICKET_INDEX_LENGTH, 0xff))
              ),
            })
            .on('data', (data: Buffer) => {
              const acknowledged = nodes[i].paymentChannels.types.AcknowledgedTicket.create(nodes[i].paymentChannels)
              acknowledged.set(data)

              tickets.push(acknowledged)
            })
            .on('end', resolve)
        )

        if (tickets.length == 0) {
          continue
        }

        console.log(tickets.length)

        for (let k = 0; k < tickets.length; k++) {
          console.log((await tickets[k].signedTicket).ticket.amount)
          // @ts-ignore
          await nodes[i].paymentChannels.channel.tickets.submit(tickets[k])
          console.log(`ticket submitted`)
        }
      }

      log(`after Promise.all`)

      await Promise.all(nodes.map((node: Hopr<HoprEthereum>) => node.stop()))
    },
    durations.seconds(25)
  )
})

/**
 * Introduce the nodes to each other
 * @param nodes Hopr nodes
 */
function connectionHelper<Chain extends HoprCoreConnector>(nodes: Hopr<Chain>[]) {
  for (let i = 0; i < nodes.length; i++) {
    for (let j = i + 1; j < nodes.length; j++) {
      nodes[i].peerStore.put(nodes[j].peerInfo)
      nodes[j].peerStore.put(nodes[i].peerInfo)
    }
  }
}

const NOOP = () => {}

function cleanUpReceiveChecker<Chain extends HoprCoreConnector>(nodes: Hopr<Chain>[]) {
  for (const node of nodes) {
    node.output = NOOP
  }
}

function receiveChecker<Chain extends HoprCoreConnector>(msgs: Uint8Array[], node: Hopr<Chain>) {
  const remainingMessages = msgs.slice()

  return new Promise<void>((resolve) => {
    node.output = (arr: Uint8Array) => {
      for (let i = 0; i < remainingMessages.length; i++) {
        if (u8aEquals(remainingMessages[i], arr)) {
          remainingMessages.splice(i, 1)
        }
      }
      if (remainingMessages.length == 0) {
        return resolve()
      }
    }
  })
}
