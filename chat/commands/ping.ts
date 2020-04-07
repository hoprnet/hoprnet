import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '../../src'
import type AbstractCommand from './abstractCommand'

import type PeerId from 'peer-id'
import type PeerInfo from 'peer-info'

import { checkPeerIdInput, isBootstrapNode } from '../utils'
import chalk from 'chalk'

export default class Ping implements AbstractCommand {
    constructor(public node: Hopr<HoprCoreConnector>) { }

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

        try {
            const latency = await this.node.ping(peerId)
            console.log(`Pong received in:`, chalk.magenta(String(latency)), `ms`)
        } catch (err) {
            console.log(`Could not ping node. Error was: ${chalk.red(err.message)}`)
        }
    }

    complete(line: string, cb: (err: Error | undefined, hits: [string[], string]) => void, query?: string): void {
        const peerInfos: PeerInfo[] = []
        for (const peerInfo of this.node.peerStore.peers.values()) {
            if ((!query || peerInfo.id.toB58String().startsWith(query)) && !isBootstrapNode(this.node, peerInfo.id)) {
                peerInfos.push(peerInfo)
            }
        }

        if (!peerInfos.length) {
            console.log(chalk.red(`\nDoesn't know any other node except apart from bootstrap node${this.node.bootstrapServers.length == 1 ? '' : 's'}!`))
            return cb(undefined, [[''], line])
        }

        return cb(undefined, [peerInfos.map((peerInfo: PeerInfo) => `ping ${peerInfo.id.toB58String()}`), line])
    }
}

