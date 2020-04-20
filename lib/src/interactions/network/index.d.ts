import HoprCoreConnector from '@hoprnet/hopr-core-connector-interface';
import Hopr from '../..';
import { Crawler } from './crawler';
import { ForwardPacketInteraction } from './forwardPacket';
import { Heartbeat } from './heartbeat';
declare class NetworkInteractions<Chain extends HoprCoreConnector> {
    crawler: Crawler<Chain>;
    forwardPacket: ForwardPacketInteraction<Chain>;
    heartbeat: Heartbeat<Chain>;
    constructor(node: Hopr<Chain>);
}
export { NetworkInteractions };
