import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface';
import type Hopr from '@hoprnet/hopr-core';
import AbstractCommand from './abstractCommand';
export default class Crawl implements AbstractCommand {
    node: Hopr<HoprCoreConnector>;
    constructor(node: Hopr<HoprCoreConnector>);
    /**
     * Crawls the network to check for other nodes. Triggered by the CLI.
     */
    execute(): Promise<void>;
    complete(line: string, cb: (err: Error | undefined, hits: [string[], string]) => void): void;
}
