import Hopr from '../..'
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import HoprEthereum from '@hoprnet/hopr-core-ethereum'
import assert from 'assert'
import { u8aEquals, durations } from '@hoprnet/hopr-utils'
import { MAX_HOPS } from '../../constants'
import LevelUp from 'levelup'
import MemDown from 'memdown'
import BN from 'bn.js'
import Debug from 'debug'
import { ACKNOWLEDGED_TICKET_INDEX_LENGTH } from '../../dbKeys'
import { connectionHelper } from '../../test-utils'

const log = Debug(`hopr-core:testing`)

const TWO_SECONDS = durations.seconds(2)
const CHANNEL_DEPOSIT = 200 // HOPRli
const BALANCE_A = 100 // HOPRli
const TICKET_AMOUNT = 10 // HOPRli
const TICKET_WIN_PROB = 1 // 100%

async function generateNode(id: number): Promise<Hopr<HoprEthereum>> {
  // Start HOPR in DEBUG_MODE and use demo seeds
  return (await Hopr.create({
    id,
    db: new LevelUp(MemDown()),
    connector: HoprEthereum,
    provider: GANACHE_URI,
    network: 'ethereum',
    debug: true,
    ticketAmount: TICKET_AMOUNT,
    ticketWinProb: TICKET_WIN_PROB
  })) as Hopr<HoprEthereum>
}

const GANACHE_URI = `ws://127.0.0.1:8545`

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

describe('packet/index.spec.ts test packet composition and decomposition', function () {
  this.timeout(30000)

  it('should create packets and decompose them', async function () {
    const nodes = await Promise.all(Array.from({ length: MAX_HOPS + 1 }).map((_value, index) => generateNode(index)))

    connectionHelper(nodes.map((n) => n._libp2p))

    new nodes[0].paymentChannels.types.ChannelBalance(undefined, {
      balance: new BN(CHANNEL_DEPOSIT),
      balance_a: new BN(BALANCE_A)
    }),
      async (bal) => {
        const channel = nodes[0].paymentChannels.types.Channel.createFunded(bal)
        const signedChannel = new nodes[0].paymentChannels.types.SignedChannel(undefined, {
          channel,
          signature: await channel.sign(nodes[0].getId().privKey.marshal(), undefined)
        } as any)
        return signedChannel
      }

    async function openChannel(a: number, b: number) {
      let channelBalance = new nodes[a].paymentChannels.types.ChannelBalance(undefined, {
        balance: new BN(CHANNEL_DEPOSIT),
        balance_a: new BN(BALANCE_A)
      })

      await nodes[a].paymentChannels.channel.create(
        nodes[b].getId().pubKey.marshal(),
        undefined, //async () => nodes[b].getId().pubKey.marshal(),
        new nodes[a].paymentChannels.types.ChannelBalance(undefined, {
          balance: new BN(CHANNEL_DEPOSIT),
          balance_a: new BN(BALANCE_A)
        }),
        (_channelBalance) => nodes[a]._interactions.payments.open.interact(nodes[b].getId(), channelBalance) as any
      )
    }

    const queries: [number, number][] = []

    for (let i = 0; i < MAX_HOPS - 1; i++) {
      for (let j = i + 1; j < MAX_HOPS; j++) {
        queries.push([i, j])
      }
    }

    await Promise.all(queries.map((query) => openChannel(query[0], query[1])))

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

    for (let i = 0; i < nodes.length; i++) {
      const tickets = []

      await new Promise((resolve) =>
        nodes[i].db
          .createValueStream({
            gte: Buffer.from(
              nodes[i]._dbKeys.AcknowledgedTickets(Buffer.alloc(ACKNOWLEDGED_TICKET_INDEX_LENGTH, 0x00))
            ),
            lt: Buffer.from(nodes[i]._dbKeys.AcknowledgedTickets(Buffer.alloc(ACKNOWLEDGED_TICKET_INDEX_LENGTH, 0xff)))
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

      for (let k = 0; k < tickets.length; k++) {
        await nodes[i].paymentChannels.channel.tickets.submit(tickets[k], undefined as any)
      }
    }

    log(`after Promise.all`)

    await Promise.all(nodes.map((node: Hopr<HoprEthereum>) => node.stop()))
  })
})
