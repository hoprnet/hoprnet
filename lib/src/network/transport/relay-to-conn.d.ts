import type { MultiaddrConnection, Stream } from './types';
import type PeerId from 'peer-id';
export declare function relayToConn(options: {
    stream: Stream;
    counterparty: PeerId;
}): MultiaddrConnection;
