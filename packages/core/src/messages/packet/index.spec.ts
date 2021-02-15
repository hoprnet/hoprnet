import Hopr from '../..'
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import HoprEthereum from '@hoprnet/hopr-core-ethereum'
import assert from 'assert'
import { u8aEquals, durations } from '@hoprnet/hopr-utils'
import { MAX_HOPS } from '../../constants'
import LevelUp from 'levelup'
import MemDown from 'memdown'
import BN from 'bn.js'

import { ACKNOWLEDGED_TICKET_INDEX_LENGTH } from '../../dbKeys'
import { connectionHelper } from '../../test-utils'
import type { AcknowledgedTicket } from '@hoprnet/hopr-core-connector-interface/src/types'
import { privKeyToPeerId } from '@hoprnet/hopr-utils'
import { NODE_SEEDS } from '@hoprnet/hopr-demo-seeds'
import type Multiaddr from 'multiaddr'

import Debug from 'debug'

const log = Debug(`hopr-core:test`)

const TWO_SECONDS = durations.seconds(2)
const CHANNEL_DEPOSIT = new BN(200) // HOPRli
const TICKET_AMOUNT = 10 // HOPRli
const TICKET_WIN_PROB = 1 // 100%

/**
 * Generates a mocked HOPR instance
 * @param id number, e.g. `0`
 * @param bootstrapNode set to true to create a bootstrap node
 * @param bootstrapServers specify a list of bootstrap server
 */
async function generateNode(
  id: number,
  bootstrapNode: boolean,
  bootstrapServers?: Multiaddr[]
): Promise<Hopr<HoprEthereum>> {
  // Start HOPR in DEBUG_MODE and use demo seeds
  return (await Hopr.create({
    id,
    peerId: await privKeyToPeerId(NODE_SEEDS[id]),
    db: new LevelUp(MemDown()),
    connector: HoprEthereum,
    provider: GANACHE_URI,
    network: 'ethereum',
    debug: true,
    ticketAmount: TICKET_AMOUNT,
    ticketWinProb: TICKET_WIN_PROB,
    bootstrapNode,
    bootstrapServers
  })) as Hopr<HoprEthereum>
}

/**
 * Fetches all tickets from the database of a node
 * @param node the HOPR instance
 */
async function getTicketsFromDatabase(node: Hopr<any>): Promise<AcknowledgedTicket[]> {
  let tickets: AcknowledgedTicket[] = []

  return new Promise((resolve, reject) =>
    node.db
      .createValueStream({
        // Note that LevelDB does not work with Uint8Array keys
        gte: Buffer.from(node._dbKeys.AcknowledgedTickets(Buffer.alloc(ACKNOWLEDGED_TICKET_INDEX_LENGTH, 0x00))),
        lt: Buffer.from(node._dbKeys.AcknowledgedTickets(Buffer.alloc(ACKNOWLEDGED_TICKET_INDEX_LENGTH, 0xff)))
      })
      // Note that LevelDB outputs Buffers and not Uint8Arrays
      .on('data', (data: Buffer) => {
        const acknowledged = node.paymentChannels.types.AcknowledgedTicket.create(node.paymentChannels)
        acknowledged.set(data)

        tickets.push(acknowledged)
      })
      .on('error', reject)
      .on('end', () => resolve(tickets))
  )
}

/**
 * Opens and funds a payment channel between `a` and `b`.
 * @param a first party
 * @param b second party
 */
async function openChannel(a: Hopr<HoprEthereum>, b: Hopr<HoprEthereum>) {
  await a.openChannel(b.getId(), CHANNEL_DEPOSIT)
}

const GANACHE_URI = `ws://127.0.0.1:8545`

const NOOP = () => {}

/**
 * Used to remove the receive checker after successfully receiving all messages
 * @param nodes node instances to clean up
 */
function cleanUpReceiveChecker<Chain extends HoprCoreConnector>(nodes: Hopr<Chain>[]) {
  for (const node of nodes) {
    node.output = NOOP
  }
}

/**
 * Attach an event listener to a node to check the reception of a message
 * @param msgs messages to check reception
 * @param node instance that should receive the messages
 */
function receiveChecker<Chain extends HoprCoreConnector>(msgs: Uint8Array[], node: Hopr<Chain>) {
  const remainingMessages = msgs.slice()

  return new Promise<void>((resolve) => {
    node.output = (arr: Uint8Array) => {
      for (let msg of remainingMessages.entries()) {
        if (u8aEquals(msg[1], arr)) {
          remainingMessages.splice(msg[0], 1)
        }
      }
      if (remainingMessages.length == 0) {
        return resolve()
      }
    }
  })
}

describe('test packet composition and decomposition', function () {
  this.timeout(60000)

  // @TODO: this needs to be reworked
  // * more documentantion
  // * ideally split this into unit tests
  // * support MAX_HOPS != 3
  it.skip('should create packets and decompose them', async function () {
    const bs = await generateNode(0, true)

    const bsAddresses = await bs.getAnnouncedAddresses()
    const nodes = await Promise.all(
      Array.from({ length: MAX_HOPS + 1 }).map((_value, index) =>
        generateNode(index + 1, false, bsAddresses.slice(1))
      )
    )

    connectionHelper(nodes.map((n: Hopr<HoprEthereum>) => n._libp2p))

    const queries: [first: number, second: number][] = []

    for (let i = 0; i < MAX_HOPS - 1; i++) {
      for (let j = i + 1; j < MAX_HOPS; j++) {
        queries.push([i, j])
      }
    }

    await Promise.all(queries.map((query) => openChannel(nodes[query[0]], nodes[query[1]])))

    const testMessages: Uint8Array[] = []

    for (let i = 0; i < MAX_HOPS; i++) {
      testMessages.push(new TextEncoder().encode(`test message #${i}`))
    }

    let msgReceivedPromises = []

    for (let i = 1; i <= MAX_HOPS; i++) {
      msgReceivedPromises.push(receiveChecker(testMessages.slice(i - 1, i), nodes[i]))
      await nodes[0].sendMessage(testMessages[i - 1], nodes[i].getId(), async () =>
        nodes.slice(1, i).map((node) => node.getId())
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
      await nodes[i].sendMessage(testMessages[i], nodes[nodes.length - 1].getId(), async () =>
        nodes.slice(i + 1, nodes.length - 1).map((node) => node.getId())
      )
    }

    await new Promise((resolve) => setTimeout(resolve, 700))

    for (let node of nodes) {
      const tickets: AcknowledgedTicket[] = await getTicketsFromDatabase(node)

      if (tickets.length == 0) {
        continue
      }

      for (let k = 0; k < tickets.length; k++) {
        await node.paymentChannels.channel.tickets.submit(tickets[k] as any, undefined as any)
        log(`ticket submitted`)
      }
    }

    await Promise.all(nodes.map((node: Hopr<HoprEthereum>) => node.stop()))
  })
})
