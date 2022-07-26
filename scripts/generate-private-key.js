import { createSecp256k1PeerId } from '@libp2p/peer-id-factory'
import { PublicKey, u8aToHex } from '@hoprnet/hopr-utils'
import { keysPBM } from '@libp2p/crypto/keys'

createSecp256k1PeerId().then((pid) => {
  console.log('PEERID: ', pid.toString())
  console.log('ADDRESS: ', PublicKey.fromPeerId(pid).toAddress().toHex())
  console.log('PRIVATE KEY:', u8aToHex(keysPBM.PrivateKey.decode(pid.privateKey).Data))
})
