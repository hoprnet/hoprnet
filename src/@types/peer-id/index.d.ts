declare module 'peer-id' {
    import c from 'libp2p-crypto'
    import CID from 'cids'

    export type PeerIdJSON = {
        id: string
        privKey: string
        pubKey: string
    }

    export default class PeerId {
        readonly id: Buffer
        privKey: any
        pubKey: any

        private _id: Buffer
        private _idB58String: string

        className: string
        symbolName: string

        constructor(id: Buffer, privKey?: any, pubKey?: any)

        // Return the protobuf version of the public key, matching go ipfs formatting
        marshalPubKey(): Buffer

        // Return the protobuf version of the private key, matching go ipfs formatting
        marshalPrivKey(): Buffer

        // Return the protobuf version of the peer-id
        marshal(excludePriv?: boolean): Buffer

        toPrint(): string

        // return the jsonified version of the key, matching the formatting
        // of go-ipfs for its config file
        toJSON(): PeerIdJSON

        // encode/decode functions
        toHexString(): string

        toBytes(): Buffer

        toB58String(): string

        // return self-describing String representation
        // in default format from RFC 0001: https://github.com/libp2p/specs/pull/209
        toString(): string

        /**
         * Checks the equality of `this` peer against a given PeerId.
         * @param {Buffer|PeerId} id
         * @returns {boolean}
         */
        equals(id: Buffer | PeerId): boolean

        /**
         * Checks the equality of `this` peer against a given PeerId.
         * @deprecated Use `.equals`
         * @param {Buffer|PeerId} id
         * @returns {boolean}
         */
        isEqual(id: Buffer | PeerId): boolean

        /*
         * Check if this PeerId instance is valid (privKey -> pubKey -> Id)
         */
        isValid(): boolean

        static create(opts?: { bits?: number; keyType?: string }): Promise<PeerId>

        static createFromHexString(str: string): PeerId

        static createFromBytes(buf: Buffer): PeerId

        static createFromB58String(str: string): PeerId

        static createFromCID(cid: CID): PeerId

        static createFromPubKey(key: Buffer | string): Promise<PeerId>

        static createFromPrivKey(key: Buffer | string): Promise<PeerId>

        static createFromJSON(obj: PeerIdJSON): Promise<PeerId>

        static createFromProtobuf(buf: Buffer | string): Promise<PeerId>

        static isPeerId(peerId: any): peerId is PeerId
    }
}
