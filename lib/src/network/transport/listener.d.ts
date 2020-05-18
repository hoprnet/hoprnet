/// <reference types="node" />
import { Server } from 'net';
import EventEmitter from 'events';
import { MultiaddrConnection, Connection, Upgrader } from './types';
import Multiaddr from 'multiaddr';
export interface Libp2pServer extends Server {
    __connections: MultiaddrConnection[];
}
export interface Listener extends EventEmitter {
    close(): void;
    listen(ma: Multiaddr): Promise<void>;
    getAddrs(): Multiaddr[];
}
export declare function createListener({ handler, upgrader }: {
    handler: (_conn: Connection) => void;
    upgrader: Upgrader;
}, options: any): Listener;
