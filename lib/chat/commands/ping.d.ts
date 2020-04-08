import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface';
import type Hopr from '../../src';
import type AbstractCommand from './abstractCommand';
export default class Ping implements AbstractCommand {
    node: Hopr<HoprCoreConnector>;
    constructor(node: Hopr<HoprCoreConnector>);
    execute(query?: string): Promise<void>;
    complete(line: string, cb: (err: Error | undefined, hits: [string[], string]) => void, query?: string): void;
}
