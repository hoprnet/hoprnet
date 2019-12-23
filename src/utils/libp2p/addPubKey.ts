import PeerId from 'peer-id'
import { keys as libp2p_crypto } from 'libp2p-crypto'
import * as Multihash from 'multihashes'

/**
 * Takes a peerId and returns a peerId with the public key set to the corresponding
 * public key.
 *
 * @param {PeerId} peerId the PeerId instance that has probably no pubKey set
 */
module.exports.addPubKey = async peerId => {
    if (PeerId.isPeerId(peerId) && peerId.pubKey) return peerId

    peerId.pubKey = await libp2p_crypto.unmarshalPublicKey(Multihash.decode(peerId.toBytes()).digest)

    return peerId
}
