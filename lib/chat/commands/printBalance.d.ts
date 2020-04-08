import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface';
import type Hopr from '../../src';
import type AbstractCommand from './abstractCommand';
export default class PrintBalance implements AbstractCommand {
    node: Hopr<HoprCoreConnector>;
    constructor(node: Hopr<HoprCoreConnector>);
    /**
     * Prints the balance of our account.
     * @notice triggered by the CLI
     */
    execute(): Promise<void>;
    complete(line: string, cb: (err: Error | undefined, hits: [string[], string]) => void): void;
}
