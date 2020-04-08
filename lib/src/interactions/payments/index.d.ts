import Hopr from '../..';
import HoprCoreConnector from '@hoprnet/hopr-core-connector-interface';
import { Opening } from './open';
import { OnChainKey } from './onChainKey';
declare class PaymentInteractions<Chain extends HoprCoreConnector> {
    open: Opening<Chain>;
    onChainKey: OnChainKey<Chain>;
    constructor(node: Hopr<Chain>);
}
export { PaymentInteractions };
