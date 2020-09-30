declare module "libp2p" {
  export default class LibP2P {
    constructor(options: any): LibP2P
    static create(options: any): any
    // @TODO add libp2p types
    emit: (event: string, ...args: any[]) => void;
    dial: (addr: Multiaddr | PeerInfo | PeerId, options?: { signal: AbortSignal }) => Promise<Handler>;
    dialProtocol: (
      addr: Multiaddr | PeerInfo | PeerId,
      protocol: string,
      options?: { signal: AbortSignal }
    ) => Promise<Handler>;
    hangUp: (addr: PeerInfo | PeerId | Multiaddr | string) => Promise<void>;
    peerInfo: PeerInfo;
    peerStore: PeerStore;
    peerRouting: {
      findPeer: (addr: PeerId) => Promise<PeerInfo>
    };
    handle: (protocol: string[], handler: (struct: { connection: any; stream: any }) => void) => void;
    on: (str: string, handler: (...props: any[]) => void) => void;
    start (): Promise<any>;
    stop (): Promise<void>;
  }
}
