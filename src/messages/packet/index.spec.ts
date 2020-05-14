// see https://github.com/trufflesuite/ganache-core/issues/465
import '@hoprnet/hopr-core-ethereum/src/ganache-core'

import Hopr from '../..'
import HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'

import HoprEthereum from '@hoprnet/hopr-core-ethereum'

// Ignore type-checking of dependencies for the moment
// @ts-ignore
const { Ganache, migrate, fund } = require('@hoprnet/hopr-ethereum')

import assert from 'assert'

import { u8aEquals, durations } from '@hoprnet/hopr-utils'

import { MAX_HOPS } from '../../constants'

import BN from 'bn.js'
import LevelUp from 'levelup'
import MemDown from 'memdown'

const WIN_PROB = new BN(1)

const TWO_SECONDS = durations.seconds(2)

// example how to use hopr-ethereum API

async function startTestnet() {
  const ganache = new Ganache()

  await ganache.start()
  await migrate()
  await fund()

  return ganache
}

async function generateNode(id: number): Promise<Hopr<HoprEthereum>> {
  const db = new LevelUp(MemDown())

  // Start HOPR in DEBUG_MODE and use demo seeds
  const node = (await Hopr.create({
    id,
    db,
    connector: HoprEthereum,
    provider: GANACHE_URI,
    network: 'ethereum',
    debug: true,
  })) as Hopr<HoprEthereum>

  return node
}

const GANACHE_URI = `ws://127.0.0.1:9545`

describe('test packet composition and decomposition', function () {
  it('should create packets and decompose them', async function () {
    this.timeout(durations.seconds(25))

    const GanacheTestnet = await startTestnet()

    const nodes = [
      await generateNode(0),
      await generateNode(1),
      await generateNode(2),
      await generateNode(3),
    ]

    connectionHelper(nodes)

    const testMessages: Uint8Array[] = []

    for (let i = 0; i <= MAX_HOPS; i++) {
      testMessages.push(new TextEncoder().encode(`test message #${i}`))
    }

    const msgReceivedPromises = []

    // for (let i = 1; i <= MAX_HOPS; i++) {
    //   msgReceivedPromises.push(receiveChecker(testMessages.slice(i - 1, i), nodes[i]))
    //   await nodes[0].sendMessage(testMessages[i - 1], nodes[i].peerInfo, async () =>
    //     nodes.slice(1, i).map(node => node.peerInfo.id)
    //   )
    // }

    // await Promise.all(msgReceivedPromises)

    msgReceivedPromises.push(receiveChecker(testMessages, nodes[nodes.length - 1]))

    for (let i = 0; i < MAX_HOPS; i++) {
      await nodes[i].sendMessage(testMessages[i], nodes[nodes.length - 1].peerInfo, async () =>
        nodes.slice(i + 1, nodes.length - 1).map(node => node.peerInfo.id)
      )
    }

    await Promise.all(msgReceivedPromises)

    const timeout = setTimeout(() => {
      assert.fail(`No message received`)
    }, TWO_SECONDS)


    clearTimeout(timeout)
  })
})

/**
 * Informs each node about the others existence.
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

function receiveChecker<Chain extends HoprCoreConnector>(msgs: Uint8Array[], node: Hopr<Chain>) {
  const remainingMessages = msgs
  return new Promise(resolve => {
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
