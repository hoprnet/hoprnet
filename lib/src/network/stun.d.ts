/// <reference types="node" />
import dgram from 'dgram';
import { HoprOptions } from '..';
export declare type Interface = {
    family: 'IPv4' | 'IPv6';
    port: number;
    address: string;
};
declare class Stun {
    private options;
    private socket;
    constructor(options: HoprOptions);
    static getExternalIP(address: {
        hostname: string;
        port: number;
    }, usePort?: number): Promise<Interface>;
    getSocket(): dgram.Socket;
    startServer(): Promise<unknown>;
    stopServer(): Promise<void>;
}
export { Stun };
