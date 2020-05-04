/// <reference types="node" />
import EventEmitter from 'events';
import Multiaddr from 'multiaddr';
export interface Listener extends EventEmitter {
    close(): void;
    listen(ma: Multiaddr): Promise<void>;
    getAddrs(): Multiaddr[];
}
export declare function createListener({ handler, upgrader }: {
    handler: any;
    upgrader: any;
}, options: any): Listener;
