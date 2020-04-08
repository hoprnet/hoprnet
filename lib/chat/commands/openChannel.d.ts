import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface';
import type AbstractCommand from './abstractCommand';
import type Hopr from '../../src';
export default class OpenChannel implements AbstractCommand {
    node: Hopr<HoprCoreConnector>;
    constructor(node: Hopr<HoprCoreConnector>);
    /**
     * Encapsulates the functionality that is executed once the user decides to open a payment channel
     * with another party.
     * @param query peerId string to send message to
     */
    execute(query?: string): Promise<void>;
    complete(line: string, cb: (err: Error | undefined, hits: [string[], string]) => void, query?: string): void;
}
