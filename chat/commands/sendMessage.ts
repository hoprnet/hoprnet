import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '../../src'
import type AbstractCommand from './abstractCommand'

import chalk from 'chalk'

import type PeerId from 'peer-id'
import type PeerInfo from 'peer-info'

import { checkPeerIdInput, encodeMessage, isBootstrapNode } from '../utils'

import readline from 'readline'

export default class SendMessage implements AbstractCommand {
    constructor(public node: Hopr<HoprCoreConnector>) {}

    /**
     * Encapsulates the functionality that is executed once the user decides to send a message.
     * @param query peerId string to send message to
     */
    async execute(rl: readline.Interface, query?: string): Promise<void> {
        if (query == null) {
            console.log(chalk.red(`Invalid arguments. Expected 'open <peerId>'. Received '${query}'`))
            return
        }

        let peerId: PeerId
        try {
            peerId = await checkPeerIdInput(query)
        } catch (err) {
            console.log(chalk.red(err.message))
            return
        }

        rl.question(`Sending message to ${chalk.blue(peerId.toB58String())}\nType in your message and press ENTER to send:\n`, async (message: string) => {
            try {
                await this.node.sendMessage(encodeMessage(message), peerId)
            } catch (err) {
                console.log(chalk.red(err.message))
            }
        })
    }

    async complete(line: string, cb: (err: Error | undefined, hits: [string[], string]) => void, query?: string): Promise<void> {
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
    
        return cb(undefined, [peerInfos.map((peerInfo: PeerInfo) => `send ${peerInfo.id.toB58String()}`), line])
    }
}


