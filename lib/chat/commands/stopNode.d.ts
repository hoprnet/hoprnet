import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface';
import type Hopr from '../../src';
import type AbstractCommand from './abstractCommand';
export default class StopNode implements AbstractCommand {
    node: Hopr<HoprCoreConnector>;
    constructor(node: Hopr<HoprCoreConnector>);
    /**
     * Stops the node and kills the process in case it does not quit by itself.
     */
    execute(): Promise<void>;
    complete(line: string, cb: (err: Error | undefined, hits: [string[], string]) => void): void;
}
