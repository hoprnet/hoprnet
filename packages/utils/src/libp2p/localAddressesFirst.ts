//const { publicAddressesFirst } = require('libp2p-utils/src/address-sort')

function localAddressesFirst (addresses) {
    return [...addresses].sort()
  }
  
module.exports.localAddressesFirst = localAddressesFirst