import libp2p = require('libp2p');
import { Network } from './network';
import { LevelUp } from 'levelup';
import Multiaddr from 'multiaddr';
import { Debugger } from 'debug';
import PeerId from 'peer-id';
import PeerInfo from 'peer-info';
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface';
import type { HoprCoreConnectorStatic } from '@hoprnet/hopr-core-connector-interface';
import { Interactions, Duplex } from './interactions';
import * as DbKeys from './dbKeys';
interface NetOptions {
    ip: string;
    port: number;
}
export declare type HoprOptions = {
    debug: boolean;
    db?: LevelUp;
    peerId?: PeerId;
    peerInfo?: PeerInfo;
    password?: string;
    id?: number;
    bootstrapNode?: boolean;
    network: string;
    connector: HoprCoreConnectorStatic;
    bootstrapServers?: PeerInfo[];
    provider: string;
    output?: (encoded: Uint8Array) => void;
    hosts?: {
        ip4?: NetOptions;
        ip6?: NetOptions;
    };
};
export default class Hopr<Chain extends HoprCoreConnector> extends libp2p {
    db: LevelUp;
    paymentChannels: Chain;
    interactions: Interactions<Chain>;
    network: Network<Chain>;
    log: Debugger;
    dbKeys: typeof DbKeys;
    output: (arr: Uint8Array) => void;
    isBootstrapNode: boolean;
    bootstrapServers: PeerInfo[];
    dial: (addr: Multiaddr | PeerInfo | PeerId, options?: {
        signal: AbortSignal;
    }) => Promise<any>;
    dialProtocol: (addr: Multiaddr | PeerInfo | PeerId, protocol: string, options?: {
        signal: AbortSignal;
    }) => Promise<{
        stream: Duplex;
        protocol: string;
    }>;
    hangUp: (addr: PeerInfo | PeerId | Multiaddr | string) => Promise<void>;
    peerInfo: PeerInfo;
    peerStore: {
        has(peerInfo: PeerId): boolean;
        put(peerInfo: PeerInfo, options?: {
            silent: boolean;
        }): PeerInfo;
        peers: Map<string, PeerInfo>;
        remove(peer: PeerId): void;
    };
    peerRouting: {
        findPeer: (addr: PeerId) => Promise<PeerInfo>;
    };
    handle: (protocol: string[], handler: (struct: {
        connection: any;
        stream: any;
    }) => void) => void;
    start: () => Promise<void>;
    stop: () => Promise<void>;
    on: (str: string, handler: (...props: any[]) => void) => void;
    /**
     * @constructor
     *
     * @param _options
     * @param provider
     */
    constructor(options: HoprOptions, db: LevelUp, paymentChannels: Chain);
    /**
     * Creates a new node.
     *
     * @param options the parameters
     */
    static create<CoreConnector extends HoprCoreConnector>(options: HoprOptions): Promise<Hopr<CoreConnector>>;
    /**
     * Parses the bootstrap servers given in `.env` and tries to connect to each of them.
     *
     * @throws an error if none of the bootstrapservers is online
     */
    connectToBootstrapServers(): Promise<void>;
    /**
     * This method starts the node and registers all necessary handlers. It will
     * also open the database and creates one if it doesn't exists.
     *
     * @param options
     */
    up(): Promise<Hopr<Chain>>;
    /**
     * Shuts down the node and saves keys and peerBook in the database
     */
    down(): Promise<void>;
    /**
     * Sends a message.
     *
     * @notice THIS METHOD WILL SPEND YOUR ETHER.
     * @notice This method will fail if there are not enough funds to open
     * the required payment channels. Please make sure that there are enough
     * funds controlled by the given key pair.
     *
     * @param msg message to send
     * @param destination PeerId of the destination
     * @param intermediateNodes optional set path manually
     * the acknowledgement of the first hop
     */
    sendMessage(msg: Uint8Array, destination: PeerId | PeerInfo, getIntermediateNodesManually?: () => Promise<PeerId[]>): Promise<void>;
    /**
     * Ping a node.
     *
     * @param destination PeerId of the node
     * @returns latency
     */
    ping(destination: PeerId): Promise<number>;
    /**
     * Takes a destination and samples randomly intermediate nodes
     * that will relay that message before it reaches its destination.
     *
     * @param destination instance of peerInfo that contains the peerId of the destination
     */
    getIntermediateNodes(destination: PeerId): Promise<PeerId[]>;
    static openDatabase(db_dir: string, constants: {
        CHAIN_NAME: string;
        NETWORK: string;
    }, options?: {
        id?: number;
        bootstrapNode?: boolean;
    }): LevelUp;
}
export {};
