import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface';
import type Hopr from '..';
import type { HoprOptions } from '..';
import { Crawler } from './crawler';
import { Heartbeat } from './heartbeat';
import { StunServer } from './stun';
declare class Network<Chain extends HoprCoreConnector> {
    crawler: Crawler<Chain>;
    heartbeat?: Heartbeat<Chain>;
    stun?: StunServer;
    constructor(node: Hopr<Chain>, options: HoprOptions);
    start(): Promise<void>;
    stop(): Promise<void>;
}
export { Network };
