/// <reference types="node" />
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface';
import type Hopr from '../../src';
import type AbstractCommand from './abstractCommand';
import readline from 'readline';
export default class SendMessage implements AbstractCommand {
    node: Hopr<HoprCoreConnector>;
    constructor(node: Hopr<HoprCoreConnector>);
    /**
     * Encapsulates the functionality that is executed once the user decides to send a message.
     * @param query peerId string to send message to
     */
    execute(rl: readline.Interface, query?: string): Promise<void>;
    complete(line: string, cb: (err: Error | undefined, hits: [string[], string]) => void, query?: string): Promise<void>;
}
