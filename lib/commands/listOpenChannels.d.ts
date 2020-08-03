import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface';
import type Hopr from '@hoprnet/hopr-core';
import AbstractCommand from './abstractCommand';
export default class ListOpenChannels implements AbstractCommand {
    node: Hopr<HoprCoreConnector>;
    constructor(node: Hopr<HoprCoreConnector>);
    /**
     * Lists all channels that we have with other nodes. Triggered from the CLI.
     */
    execute(): Promise<void>;
    complete(line: string, cb: (err: Error | undefined, hits: [string[], string]) => void): void;
}
