import type Hopr from '../../';
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface';
import type { Types } from '@hoprnet/hopr-core-connector-interface';
import type { AbstractInteraction } from '../abstractInteraction';
import type { Handler } from '../../network/transport/types';
import PeerInfo from 'peer-info';
import type PeerId from 'peer-id';
declare class Opening<Chain extends HoprCoreConnector> implements AbstractInteraction<Chain> {
    node: Hopr<Chain>;
    protocols: string[];
    constructor(node: Hopr<Chain>);
    handler(struct: Handler): Promise<void>;
    interact(counterparty: PeerInfo | PeerId, channelBalance: Types.ChannelBalance): Promise<Types.SignedChannel<Types.Channel, Types.Signature>>;
    private collect;
}
export { Opening };
