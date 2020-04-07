import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type { Types, Channel as ChannelInstance } from '@hoprnet/hopr-core-connector-interface'
import type AbstractCommand from './abstractCommand'

import BN from 'bn.js'

import type Hopr from '../../src'

import type PeerId from 'peer-id'

import chalk from 'chalk'

import { checkPeerIdInput, startDelayedInterval, isBootstrapNode } from '../utils'
import { u8aToHex, pubKeyToPeerId } from '../../src/utils'

export default class OpenChannel implements AbstractCommand {
    constructor(public node: Hopr<HoprCoreConnector>) { }

    /**
     * Encapsulates the functionality that is executed once the user decides to open a payment channel
     * with another party.
     * @param query peerId string to send message to
     */
    async execute(query?: string): Promise<void> {
        if (query == null || query == '') {
            console.log(chalk.red(`Invalid arguments. Expected 'open <peerId>'. Received '${query}'`))
            return
        }

        let counterparty: PeerId
        try {
            counterparty = await checkPeerIdInput(query)
        } catch (err) {
            console.log(err.message)
            return
        }

        const channelId = await this.node.paymentChannels.utils.getId(
            /* prettier-ignore */
            await this.node.paymentChannels.utils.pubKeyToAccountId(this.node.peerInfo.id.pubKey.marshal()),
            await this.node.paymentChannels.utils.pubKeyToAccountId(counterparty.pubKey.marshal())
        )

        const unsubscribe = startDelayedInterval(`Submitted transaction. Waiting for confirmation`)

        try {
            await this.node.paymentChannels.channel.create(
                this.node.paymentChannels,
                counterparty.pubKey.marshal(),
                async () => this.node.paymentChannels.utils.pubKeyToAccountId(await this.node.interactions.payments.onChainKey.interact(counterparty)),
                this.node.paymentChannels.types.ChannelBalance.create(undefined, {
                    balance: new BN(12345),
                    balance_a: new BN(123)
                }),
                (balance: Types.ChannelBalance): Promise<Types.SignedChannel<Types.Channel, Types.Signature>> => this.node.interactions.payments.open.interact(counterparty, balance)
            )

            console.log(`${chalk.green(`Successfully opened channel`)} ${chalk.yellow(u8aToHex(channelId))}`)
        } catch (err) {
            console.log(chalk.red(err.message))
        }

        await new Promise(resolve => setTimeout(() => {
            unsubscribe()
            resolve()
        }))
    }

    complete(line: string, cb: (err: Error | undefined, hits: [string[], string]) => void, query?: string) {
        this.node.paymentChannels.channel.getAll(
            this.node.paymentChannels,
            async (channel: ChannelInstance<HoprCoreConnector>) => (await pubKeyToPeerId(await channel.offChainCounterparty)).toB58String(),
            async (channelIds: Promise<string>[]) => {
                let peerIdStringSet: Set<string>

                try {
                    peerIdStringSet = new Set<string>(await Promise.all(channelIds))
                } catch (err) {
                    console.log(chalk.red(err.message))
                    return cb(undefined, [[''], line])
                }

                const peers: string[] = []
                for (const peerInfo of this.node.peerStore.peers.values()) {
                    if (isBootstrapNode(this.node, peerInfo.id)) {
                        continue
                    }

                    if (!peerIdStringSet.has(peerInfo.id.toB58String())) {
                        peers.push(peerInfo.id.toB58String())
                    }
                }

                if (peers.length < 1) {
                    console.log(chalk.red(`\nDoesn't know any node to open a payment channel with.`))
                    return cb(undefined, [[''], line])
                }

                const hits = query ? peers.filter((peerId: string) => peerId.startsWith(query)) : peers

                return cb(undefined, [hits.length ? hits.map((str: string) => `open ${str}`) : ['open'], line])
            })
    }
}