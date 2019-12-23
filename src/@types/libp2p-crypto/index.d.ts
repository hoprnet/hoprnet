declare module 'libp2p-crypto' {
    import secp256k1 from 'libp2p-crypto-secp256k1'
    export namespace keys {
        // export type keyType
        export const supportedKeys: {
            rsa: any
            ed25519: any
            secp256k1: secp256k1
        }

        export const keysPBM: any

        export function generateKeyPair(type: any, bits: any): Promise<any>

        export function generateKeyPairFromSeed(type: any, seed: any, bits: any): Promise<any>

        export function keyStretcher(cipherType: any, hash: any, secret: any): Promise<any>
        export function generateEphemeralKeyPair(curve): Promise<any>

        export function unmarshalPublicKey(buf: Buffer): Promise<any | secp256k1.Secp256k1PublicKey>

        export function marshalPublicKey(key: any, type: any): Buffer

        export function unmarshalPrivateKey(buf: Buffer): Promise<secp256k1.Secp256k1PrivateKey>

        export function marshalPrivateKey(key: any, type: any): Buffer
    }
}
