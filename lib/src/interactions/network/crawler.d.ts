import type Hopr from '../../';
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface';
import type { AbstractInteraction } from '../abstractInteraction';
import type PeerInfo from 'peer-info';
declare class Crawler<Chain extends HoprCoreConnector> implements AbstractInteraction<Chain> {
    node: Hopr<Chain>;
    protocols: string[];
    constructor(node: Hopr<Chain>);
    handler(struct: {
        stream: any;
    }): void;
    interact(counterparty: PeerInfo): Promise<PeerInfo[]>;
}
export { Crawler };
