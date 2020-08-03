import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import type AbstractCommand from './abstractCommand'

import type PeerId from 'peer-id'

import { checkPeerIdInput, isBootstrapNode, getPeers } from '../utils'
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
    const peers = getPeers(this.node)

    const peerIds =
      !query || query.length == 0
        ? peers.map((peer) => peer.toB58String())
        : peers.reduce((acc: string[], peer: PeerId) => {
            const peerString = peer.toB58String()
            if (peerString.startsWith(query)) {
              acc.push(peerString)
            }

            return acc
          }, [])

    if (!peerIds.length) {
      return cb(undefined, [[''], line])
    }

    return cb(undefined, [peerIds.map((peerId: string) => `ping ${peerId}`), line])
  }
}
