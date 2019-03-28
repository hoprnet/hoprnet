'use strict'

module.exports.PRIVATE_KEY_LENGTH = 32
module.exports.HASH_LENGTH = 32
module.exports.KEY_LENGTH = this.HASH_LENGTH
module.exports.COMPRESSED_PUBLIC_KEY_LENGTH = 33
module.exports.ADDRESS_SIZE = this.COMPRESSED_PUBLIC_KEY_LENGTH
module.exports.DESINATION_SIZE = this.ADDRESS_SIZE
module.exports.MAC_SIZE = 32
module.exports.PROVING_VALUES_SIZE = this.HASH_LENGTH + this.KEY_LENGTH
module.exports.IDENTIFIER_SIZE = 16


module.exports.PER_HOP_SIZE = this.ADDRESS_SIZE + this.MAC_SIZE + this.PROVING_VALUES_SIZE
module.exports.LAST_HOP_SIZE = this.DESINATION_SIZE + this.IDENTIFIER_SIZE