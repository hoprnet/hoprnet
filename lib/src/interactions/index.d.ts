import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface';
import type Hopr from '..';
import { PaymentInteractions } from './payments';
import { NetworkInteractions } from './network';
import { PacketInteractions } from './packet';
declare class Interactions<Chain extends HoprCoreConnector> {
    payments: PaymentInteractions<Chain>;
    network: NetworkInteractions<Chain>;
    packet: PacketInteractions<Chain>;
    constructor(node: Hopr<Chain>);
}
export { Interactions };
