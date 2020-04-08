/// <reference types="node" />
import { Header } from './index';
import HoprCoreConnector from '@hoprnet/hopr-core-connector-interface';
import Hopr from '../../../';
import PeerId from 'peer-id';
export declare function createHeader<Chain extends HoprCoreConnector>(node: Hopr<Chain>, header: Header<Chain>, peerIds: PeerId[]): Promise<{
    header: Header<Chain>;
    secrets: Uint8Array[];
    identifier: Buffer;
}>;
