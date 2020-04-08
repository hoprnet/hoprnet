import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface';
import type Hopr from '..';
import { Crawler } from './crawler';
import { Heartbeat } from './heartbeat';
declare class Network<Chain extends HoprCoreConnector> {
    crawler: Crawler<Chain>;
    heartbeat: Heartbeat<Chain>;
    constructor(node: Hopr<Chain>);
}
export { Network };
