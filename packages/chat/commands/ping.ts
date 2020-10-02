import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import type PeerId from 'peer-id'
import type { AutoCompleteResult } from './abstractCommand'
import chalk from 'chalk'
import { checkPeerIdInput, isBootstrapNode, getPeers } from '../utils'
import { AbstractCommand } from './abstractCommand'

export default class Ping extends AbstractCommand {
  constructor(public node: Hopr<HoprCoreConnector>) {
    super()
  }

  public name() {
    return 'ping'
  }

  public help() {
    return 'pings another node to check its availability'
  }

  public async execute(query?: string): Promise<string> {
    if (query == null) {
      return `Invalid arguments. Expected 'ping <peerId>'. Received '${query}'`
    }

    let peerId: PeerId
    try {
      peerId = await checkPeerIdInput(query)
    } catch (err) {
      return chalk.red(err.message)
    }

    let out = ''
    if (isBootstrapNode(this.node, peerId)) {
      out += chalk.gray(`Pinging the bootstrap node ...`) + '\n'
    }

    try {
      const latency = await this.node.ping(peerId)
      return `${out}Pong received in: ${chalk.magenta(String(latency))}ms`
    } catch (err) {
      return `${out}Could not ping node. Error was: ${chalk.red(err.message)}`
    }
  }

  public async autocomplete(query: string, line: string): Promise<AutoCompleteResult> {
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
      return [[''], line]
    }

    return [peerIds.map((peerId: string) => `ping ${peerId}`), line]
  }
}
