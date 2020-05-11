/// <reference types="node" />
import { MultiaddrConnection } from './types';
import type Multiaddr from 'multiaddr';
import type { Socket } from 'net';
export declare function socketToConn(socket: Socket, options?: {
    listeningAddr?: Multiaddr;
    localAddr?: Multiaddr;
    remoteAddr?: Multiaddr;
    signal?: AbortSignal;
}): MultiaddrConnection;
