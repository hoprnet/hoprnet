import type Multiaddr from 'multiaddr';
import PeerInfo from 'peer-info';
import PeerId from 'peer-id';
interface DialOptions {
    signal?: AbortSignal;
    relay?: PeerInfo | PeerId;
}
export interface MultiaddrConnection {
    sink(source: any): void;
    source: any;
    close(err?: Error): Promise<void>;
    conn: any;
    remoteAddr: Multiaddr;
    localAddr?: Multiaddr;
    timeline: {
        open: number;
        close?: number;
    };
}
export interface Upgrader {
    upgradeOutbound(multiaddrConnection: MultiaddrConnection): Promise<any>;
    upgradeInbound(multiaddrConnection: MultiaddrConnection): Promise<any>;
}
/**
 * @class TCP
 */
declare class TCP {
    private _upgrader;
    constructor({ upgrader }: {
        upgrader: Upgrader;
    });
    /**
     * @async
     * @param {Multiaddr} ma
     * @param {object} options
     * @param {AbortSignal} options.signal Used to abort dial requests
     * @returns {Connection} An upgraded Connection
     */
    dial(ma: Multiaddr, options?: DialOptions): Promise<any>;
    /**
     * @private
     * @param {Multiaddr} ma
     * @param {object} options
     * @param {AbortSignal} options.signal Used to abort dial requests
     * @returns {Promise<Socket>} Resolves a TCP Socket
     */
    _connect(ma: Multiaddr, options: DialOptions): Promise<unknown>;
    /**
     * Creates a TCP listener. The provided `handler` function will be called
     * anytime a new incoming Connection has been successfully upgraded via
     * `upgrader.upgradeInbound`.
     * @param {*} [options]
     * @param {function(Connection)} handler
     * @returns {Listener} A TCP listener
     */
    createListener(options: any, handler: any): import("./listener").Listener;
    /**
     * Takes a list of `Multiaddr`s and returns only valid TCP addresses
     * @param {Multiaddr[]} multiaddrs
     * @returns {Multiaddr[]} Valid TCP multiaddrs
     */
    filter(multiaddrs: any): any;
}
export default TCP;
