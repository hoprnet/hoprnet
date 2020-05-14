/// <reference types="node" />
import EventEmitter from 'events';
import { Connection, Upgrader } from './types';
import Multiaddr from 'multiaddr';
export interface Listener extends EventEmitter {
    close(): void;
    listen(ma: Multiaddr): Promise<void>;
    getAddrs(): Multiaddr[];
}
export declare function createListener({ handler, upgrader }: {
    handler: (_conn: Connection) => void;
    upgrader: Upgrader;
}, options: any): Listener;
