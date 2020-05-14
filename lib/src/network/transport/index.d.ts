/// <reference types="node" />
import type { Socket } from 'net';
import libp2p = require('libp2p');
import { Listener } from './listener';
import Multiaddr from 'multiaddr';
import PeerInfo from 'peer-info';
import PeerId from 'peer-id';
import type { Connection, Upgrader, DialOptions, Handler } from './types';
/**
 * @class TCP
 */
declare class TCP {
    get [Symbol.toStringTag](): string;
    private _upgrader;
    private _dialer;
    private _registrar;
    private _peerInfo;
    private _handle;
    private _unhandle;
    private relay?;
    private connHandler;
    constructor({ upgrader, libp2p, bootstrap }: {
        upgrader: Upgrader;
        libp2p: libp2p;
        bootstrap?: PeerInfo;
    });
    private relayToConn;
    deliveryHandlerFactory(sender: PeerId): (handler: Handler) => void;
    forwardHandlerFactory(counterparty: PeerId): (handler: Handler) => void;
    handleDeliveryRegister({ stream }: Handler): void;
    handleRelayUnregister({ stream, connection }: Handler): void;
    closeConnection(counterparty: PeerId): Promise<void>;
    registerDelivery(outerconnection: Connection, counterparty: PeerId): Promise<Uint8Array>;
    handleRelayRegister({ stream, connection }: Handler): void;
    /**
     * @async
     * @param {Multiaddr} ma
     * @param {object} options
     * @param {AbortSignal} options.signal Used to abort dial requests
     * @returns {Connection} An upgraded Connection
     */
    dial(ma: Multiaddr, options?: DialOptions): Promise<Connection>;
    dialWithRelay(ma: Multiaddr, options?: DialOptions): Promise<Connection>;
    dialDirectly(ma: Multiaddr, options?: DialOptions): Promise<Connection>;
    /**
     * @private
     * @param {Multiaddr} ma
     * @param {object} options
     * @param {AbortSignal} options.signal Used to abort dial requests
     * @returns {Promise<Socket>} Resolves a TCP Socket
     */
    _connect(ma: Multiaddr, options: DialOptions): Promise<Socket>;
    /**
     * Creates a TCP listener. The provided `handler` function will be called
     * anytime a new incoming Connection has been successfully upgraded via
     * `upgrader.upgradeInbound`.
     * @param {*} [options]
     * @param {function(Connection)} handler
     * @returns {Listener} A TCP listener
     */
    createListener(options: any, handler: (connection: any) => void): Listener;
    /**
     * Takes a list of `Multiaddr`s and returns only valid TCP addresses
     * @param multiaddrs
     * @returns Valid TCP multiaddrs
     */
    filter(multiaddrs: Multiaddr[]): Multiaddr[];
}
export default TCP;
