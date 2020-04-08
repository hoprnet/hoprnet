import type Hopr from '../../';
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface';
import type { AbstractInteraction } from '../abstractInteraction';
import PeerInfo from 'peer-info';
import type PeerId from 'peer-id';
declare class OnChainKey<Chain extends HoprCoreConnector> implements AbstractInteraction<Chain> {
    node: Hopr<Chain>;
    protocols: string[];
    constructor(node: Hopr<Chain>);
    handler(struct: {
        stream: any;
    }): void;
    interact(counterparty: PeerInfo | PeerId): Promise<Uint8Array>;
}
export { OnChainKey };
