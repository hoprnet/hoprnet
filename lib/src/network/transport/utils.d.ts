import Multiaddr from 'multiaddr';
declare type MultiaddrFamily = 'ip4' | 'ip6';
export declare function multiaddrToNetConfig(addr: Multiaddr): string | {
    family: "ipv4" | "ipv6";
    host: string;
    transport: RTCIceProtocol;
    port: number;
};
export declare function getMultiaddrs(proto: MultiaddrFamily, ip: string, port: number): Multiaddr[];
export declare function isAnyAddr(ip: string): boolean;
export {};
