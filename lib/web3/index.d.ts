/// <reference types="node" />
import EventEmitter from 'events';
import Web3 from 'web3';
export declare enum Events {
    'connected' = "connected",
    'disconnected' = "disconnected",
    'reconnected' = "reconnected"
}
export declare type IEvents = keyof typeof Events;
export declare const isConnectionError: (err: Error) => boolean;
interface IEventEmitter extends EventEmitter {
    on(event: IEvents, listener: () => void): this;
    off(event: IEvents, listener: () => void): this;
    once(event: IEvents, listener: () => void): this;
    emit(event: IEvents): boolean;
}
declare class CustomWeb3 extends Web3 implements Web3 {
    private readonly uri;
    private readonly ops;
    private reconnecting;
    private manualDisconnect;
    events: IEventEmitter;
    constructor(uri: string, ops?: {
        reconnection: boolean;
        reconnectionDelay: number;
    });
    private disconnected;
    private reconnect;
    isConnected(): Promise<boolean>;
    connect(): Promise<boolean>;
    disconnect(): Promise<void>;
}
export default CustomWeb3;
