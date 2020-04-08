import type Hopr from '../..';
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface';
import { PacketForwardInteraction } from './forward';
import { PacketAcknowledgementInteraction } from './acknowledgement';
declare class PacketInteractions<Chain extends HoprCoreConnector> {
    acknowledgment: PacketAcknowledgementInteraction<Chain>;
    forward: PacketForwardInteraction<Chain>;
    constructor(node: Hopr<Chain>);
}
export { PacketInteractions };
