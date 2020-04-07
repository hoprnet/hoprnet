import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '../../src'
import type AbstractCommand from './abstractCommand'

import chalk from 'chalk'

export default class PrintBalance implements AbstractCommand {
    constructor(public node: Hopr<HoprCoreConnector>) { }

    /**
     * Prints the balance of our account.
     * @notice triggered by the CLI
     */
    async execute(): Promise<void> {
        // @TODO replace HOPR tokens by TOKEN_NAME
        console.log(`Account Balance:  `, chalk.magenta((await this.node.paymentChannels.accountBalance).toString()), `HOPR tokens`)
    }

    complete(line: string, cb: (err: Error | undefined, hits: [string[], string]) => void): void {
        cb(undefined, [[''], line])
    }
}
