import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface';
import type Hopr from '..';
import type { HoprOptions } from '..';
import { Crawler } from './crawler';
import { Heartbeat } from './heartbeat';
import { Stun } from './stun';
declare class Network<Chain extends HoprCoreConnector> {
    private options;
    crawler: Crawler<Chain>;
    heartbeat: Heartbeat<Chain>;
    stun?: Stun;
    constructor(node: Hopr<Chain>, options: HoprOptions);
    start(): Promise<void>;
    stop(): Promise<void>;
}
export { Network };
