var PeerId = require('peer-id')

PeerId.create({ keyType: 'secp256k1' }).then((pid) => {
  console.log(pid.privKey.marshal().toString('hex'))
})
