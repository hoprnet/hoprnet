import type Hopr from '../../';
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface';
import type { AbstractInteraction } from '../abstractInteraction';
import PeerInfo from 'peer-info';
import PeerId from 'peer-id';
declare class ForwardPacketInteraction<Chain extends HoprCoreConnector> implements AbstractInteraction<Chain> {
    node: Hopr<Chain>;
    protocols: string[];
    private tokens;
    private queue;
    private promises;
    private connectionEnds;
    constructor(node: Hopr<Chain>);
    handler(struct: {
        connection: any;
        stream: any;
    }): void;
    handleForwardPacket(token: number): Promise<void>;
    interact(counterparty: PeerInfo | PeerId, relay: PeerId | PeerInfo): Promise<any>;
}
export { ForwardPacketInteraction };
