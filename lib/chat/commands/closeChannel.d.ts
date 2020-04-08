import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface';
import type AbstractCommand from './abstractCommand';
import type Hopr from '../../src';
export default class CloseChannel implements AbstractCommand {
    node: Hopr<HoprCoreConnector>;
    constructor(node: Hopr<HoprCoreConnector>);
    execute(query?: string): Promise<any>;
    complete(line: string, cb: (err: Error | undefined, hits: [string[], string]) => void, query?: string): void;
}
