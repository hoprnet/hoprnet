import assert from 'assert'
import PeerInfo from 'peer-info'
import PeerId from 'peer-id'

// @ts-ignore
import libp2p = require('libp2p')
// @ts-ignore
import TCP = require('libp2p-tcp')
// @ts-ignore
import MPLEX = require('libp2p-mplex')
// @ts-ignore
import SECIO = require('libp2p-secio')

import Debug from 'debug'
import chalk from 'chalk'

import Hopr from '..'
import HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import { Interactions } from '../interactions'
import { Crawler } from './crawler'
import { Crawler as CrawlerInteraction } from '../interactions/network/crawler'
import Multiaddr from 'multiaddr'

describe('test crawler', function () {
  async function generateNode(): Promise<Hopr<HoprCoreConnector>> {
    const node = (await libp2p.create({
      peerInfo: await PeerInfo.create(await PeerId.create({ keyType: 'secp256k1' })),
      modules: {
        transport: [TCP],
        streamMuxer: [MPLEX],
        connEncryption: [SECIO],
      },
    })) as Hopr<HoprCoreConnector>

    node.peerInfo.multiaddrs.add(Multiaddr('/ip4/0.0.0.0/tcp/0'))

    await node.start()

    node.peerRouting.findPeer = (_: PeerId) => Promise.reject('not implemented')

    node.interactions = {
      network: {
        crawler: new CrawlerInteraction(node),
      },
    } as Hopr<HoprCoreConnector>['interactions']

    new Interactions(node)
    node.network = {
      crawler: new Crawler(node),
    } as Hopr<HoprCoreConnector>['network']

    node.log = Debug(`${chalk.blue(node.peerInfo.id.toB58String())}: `)

    return (node as unknown) as Hopr<HoprCoreConnector>
  }

  it('should crawl the network and find some nodes', async function () {
    const [Alice, Bob, Chris, Dave, Eve] = await Promise.all([
      generateNode(),
      generateNode(),
      generateNode(),
      generateNode(),
      generateNode(),
    ])

    await assert.rejects(
      () => Alice.network.crawler.crawl(),
      Error(`Unable to find enough other nodes in the network.`)
    )

    Alice.peerStore.put(Bob.peerInfo)

    await assert.rejects(
      () => Alice.network.crawler.crawl(),
      Error(`Unable to find enough other nodes in the network.`)
    )

    Bob.peerStore.put(Chris.peerInfo)

    await assert.rejects(
      () => Alice.network.crawler.crawl(),
      Error(`Unable to find enough other nodes in the network.`)
    )

    Chris.peerStore.put(Dave.peerInfo)

    await assert.doesNotReject(() => Alice.network.crawler.crawl(), `Should find enough nodes.`)

    Bob.peerStore.put(Alice.peerInfo)
    Dave.peerStore.put(Eve.peerInfo)

    await assert.doesNotReject(() => Bob.network.crawler.crawl(), `Should find enough nodes.`)

    await Promise.all([
      /* prettier-ignore */
      Alice.stop(),
      Bob.stop(),
      Chris.stop(),
      Dave.stop(),
      Eve.stop(),
    ])
  })
})
