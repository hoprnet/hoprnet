"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.LAST_HOP_SIZE = exports.PER_HOP_SIZE = exports.IDENTIFIER_SIZE = exports.PROVING_VALUES_SIZE = exports.MAC_SIZE = exports.DESINATION_SIZE = exports.ADDRESS_SIZE = exports.COMPRESSED_PUBLIC_KEY_LENGTH = exports.KEY_LENGTH = exports.HASH_LENGTH = exports.PRIVATE_KEY_LENGTH = void 0;
exports.PRIVATE_KEY_LENGTH = 32;
exports.HASH_LENGTH = 32;
exports.KEY_LENGTH = exports.HASH_LENGTH;
exports.COMPRESSED_PUBLIC_KEY_LENGTH = 33;
exports.ADDRESS_SIZE = exports.COMPRESSED_PUBLIC_KEY_LENGTH;
exports.DESINATION_SIZE = exports.ADDRESS_SIZE;
exports.MAC_SIZE = 32;
exports.PROVING_VALUES_SIZE = exports.KEY_LENGTH + exports.KEY_LENGTH;
exports.IDENTIFIER_SIZE = 16;
exports.PER_HOP_SIZE = exports.ADDRESS_SIZE + exports.MAC_SIZE + exports.PROVING_VALUES_SIZE;
exports.LAST_HOP_SIZE = exports.DESINATION_SIZE + exports.IDENTIFIER_SIZE;
//# sourceMappingURL=parameters.js.map