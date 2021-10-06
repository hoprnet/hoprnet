var PeerId = require('peer-id')
var PublicKey = require('../packages/utils').PublicKey

PeerId.create({ keyType: 'secp256k1' }).then((pid) => {
  console.log("PEERID: ", pid.toB58String())
  console.log("ADDRESS: ", PublicKey.fromPeerId(pid).toAddress().toHex())
  console.log('PRIVATE KEY:', pid.privKey.marshal().toString('hex'))
})
