declare module 'peer-info' {
    import PeerId from 'peer-id'
    import * as Multiaddr from 'multiaddr'

    // Peer represents a peer on the IPFS network
    export default class PeerInfo {
        public id: PeerId
        public multiaddrs: MultiaddrSet

        private _connectedMultiaddr?: Multiaddr

        /**
         * Stores protocols this peers supports
         */
        public protocols: Set<string>

        constructor(peerId: PeerId)

        // only stores the current multiaddr being used
        connect(ma: Multiaddr): void

        disconnect(): void
        isConnected(): Multiaddr | undefined

        static isPeerInfo(peerInfo: any): peerInfo is PeerInfo

        static create(peerId?: PeerId): Promise<PeerInfo>
    }

    // Because JavaScript doesn't let you overload the compare in Set()..
    export class MultiaddrSet {
        private _multiaddrs?: Multiaddr[]
        private _observedMultiaddrs?: Multiaddr[]

        constructor(multiaddrs?: Multiaddr[])

        readonly size: number

        add(ma: Multiaddr): void

        // addSafe - prevent multiaddr explosion™
        // Multiaddr explosion is when you dial to a bunch of nodes and every node
        // gives you a different observed address and you start storing them all to
        // share with other peers. This seems like a good idea until you realize that
        // most of those addresses are unique to the subnet that peer is in and so,
        // they are completely worthless for all the other peers. This method is
        // exclusively used by identify.
        addSafe(ma: Multiaddr): void

        toArray(): Multiaddr[]

        forEach(fn: (ma: MultiaddrSet, index: number, array: Multiaddr[]) => void): void

        filterBy(maFmt: { matches: (ma: Multiaddr) => boolean; partialMatch: (ma: Multiaddr) => boolean; toString: () => string }): Multiaddr[]

        has(ma: Multiaddr): boolean

        delete(ma: Multiaddr): void

        // replaces selected existing multiaddrs with new ones
        replace(existing: Multiaddr | Multiaddr[], fresh: Multiaddr | Multiaddr[]): void

        clear(): void

        // this only really helps make ip6 and ip4 multiaddrs distinct if they are
        // different
        // TODO this is not an ideal solution, probably this code should just be
        // in libp2p-tcp
        distinct(): Multiaddr[]
    }
}
