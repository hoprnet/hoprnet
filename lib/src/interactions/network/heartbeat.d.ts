import type Hopr from '../../';
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface';
import type { AbstractInteraction } from '../abstractInteraction';
import PeerInfo from 'peer-info';
import type PeerId from 'peer-id';
declare class Heartbeat<Chain extends HoprCoreConnector> implements AbstractInteraction<Chain> {
    node: Hopr<Chain>;
    protocols: string[];
    constructor(node: Hopr<Chain>);
    handler(struct: {
        connection: any;
        stream: any;
    }): void;
    interact(counterparty: PeerInfo | PeerId): Promise<void>;
}
export { Heartbeat };
