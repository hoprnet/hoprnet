import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '../../src'
import type AbstractCommand from './abstractCommand'

import chalk from 'chalk'
import BN from 'bn.js'

export default class PrintBalance implements AbstractCommand {
    constructor(public node: Hopr<HoprCoreConnector>) { }

    /**
     * Prints the balance of our account.
     * @notice triggered by the CLI
     */
    async execute(): Promise<void> {
        console.log(
            `Account Balance: ${
                chalk.magenta((await this.node.paymentChannels.accountBalance).div(new BN(10).pow(new BN(this.node.paymentChannels.types.Balance.DECIMALS))).toString())} ${this.node.paymentChannels.types.Balance.SYMBOL}`
        )
    }

    complete(line: string, cb: (err: Error | undefined, hits: [string[], string]) => void): void {
        cb(undefined, [[''], line])
    }
}
