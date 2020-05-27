import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import type AbstractCommand from './abstractCommand'

import type PeerId from 'peer-id'

import { checkPeerIdInput, isBootstrapNode } from '../utils'
import chalk from 'chalk'

export default class Ping implements AbstractCommand {
  constructor(public node: Hopr<HoprCoreConnector>) {}

  async execute(query?: string): Promise<void> {
    if (query == null) {
      console.log(chalk.red(`Invalid arguments. Expected 'ping <peerId>'. Received '${query}'`))
      return
    }

    let peerId: PeerId
    try {
      peerId = await checkPeerIdInput(query)
    } catch (err) {
      console.log(chalk.red(err.message))
      return
    }

    if (isBootstrapNode(this.node, peerId)) {
      console.log(chalk.gray(`Pinging the bootstrap node ...`))
    }

    try {
      const latency = await this.node.ping(peerId)
      console.log(`Pong received in:`, chalk.magenta(String(latency)), `ms`)
    } catch (err) {
      console.log(`Could not ping node. Error was: ${chalk.red(err.message)}`)
    }
  }

  complete(line: string, cb: (err: Error | undefined, hits: [string[], string]) => void, query?: string): void {
    const peerInfos: string[] = []
    for (const peerInfo of this.node.peerStore.peers.values()) {
      if (!query || peerInfo.id.toB58String().startsWith(query)) {
        peerInfos.push(peerInfo.id.toB58String())
      }
    }

    if (!peerInfos.length) {
      return cb(undefined, [[''], line])
    }

    return cb(undefined, [peerInfos.map((peerInfo: string) => `ping ${peerInfo}`), line])
  }
}
