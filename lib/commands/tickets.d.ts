import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface';
import type Hopr from '@hoprnet/hopr-core';
import type AbstractCommand from './abstractCommand';
export default class Tickets implements AbstractCommand {
    node: Hopr<HoprCoreConnector>;
    constructor(node: Hopr<HoprCoreConnector>);
    /**
     * @param query channelId string to send message to
     */
    execute(query?: string): Promise<void>;
    complete(line: string, cb: (err: Error | undefined, hits: [string[], string]) => void, query?: string): void;
}
