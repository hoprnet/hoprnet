import PeerId from 'peer-id'
import { keys as libp2p_crypto } from 'libp2p-crypto'

const PRIVKEY_LENGTH = 32

/**
 * Converts a plain compressed ECDSA private key over the curve `secp256k1`
 * to a peerId in order to use it with libp2p.
 * It equips the generated peerId with private key and public key.
 *
 * @param {Buffer | string} privKey the plain private key
 */
export default function privKeyToPeerId(privKey: Buffer | string): Promise<PeerId> {
    if (typeof privKey === 'string') {
        privKey = Buffer.from(privKey.replace(/0x/, ''), 'hex')
    }

    if (!Buffer.isBuffer(privKey)) {
        throw Error(`Unable to parse private key to desired representation. Got type '${typeof privKey}'.`)
    }

    if (privKey.length != PRIVKEY_LENGTH) {
        throw Error(`Invalid private key. Expected a buffer of size ${PRIVKEY_LENGTH} bytes. Got one of ${privKey.length} bytes.`)
    }

    const secp256k1PrivKey = new libp2p_crypto.supportedKeys.secp256k1.Secp256k1PrivateKey(privKey)

    return PeerId.createFromPrivKey(secp256k1PrivKey.bytes)
}
