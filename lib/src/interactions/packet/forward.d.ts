import { Packet } from '../../messages/packet';
import type PeerId from 'peer-id';
import PeerInfo from 'peer-info';
import type { AbstractInteraction } from '../abstractInteraction';
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface';
import type Hopr from '../../';
import type { Handler } from '../../network/transport/types';
declare class PacketForwardInteraction<Chain extends HoprCoreConnector> implements AbstractInteraction<Chain> {
    node: Hopr<Chain>;
    private tokens;
    private queue;
    private promises;
    protocols: string[];
    constructor(node: Hopr<Chain>);
    interact(counterparty: PeerInfo | PeerId, packet: Packet<Chain>): Promise<void>;
    handler(struct: Handler): void;
    handlePacket(token: number): Promise<void>;
}
export { PacketForwardInteraction };
