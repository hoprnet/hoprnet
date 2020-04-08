/// <reference types="node" />
import { AbstractInteraction } from '../abstractInteraction';
import PeerId from 'peer-id';
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface';
import type Hopr from '../../';
import { Acknowledgement } from '../../messages/acknowledgement';
import EventEmitter from 'events';
declare class PacketAcknowledgementInteraction<Chain extends HoprCoreConnector> extends EventEmitter implements AbstractInteraction<Chain> {
    node: Hopr<Chain>;
    protocols: string[];
    constructor(node: Hopr<Chain>);
    handler(struct: {
        stream: any;
    }): void;
    interact(counterparty: PeerId, acknowledgement: Acknowledgement<Chain>): Promise<void>;
}
export { PacketAcknowledgementInteraction };
