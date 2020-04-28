"use strict";
function __export(m) {
    for (var p in m) if (!exports.hasOwnProperty(p)) exports[p] = m[p];
}
Object.defineProperty(exports, "__esModule", { value: true });
__export(require("./deserialize"));
__export(require("./serialize"));
exports.KEYPAIR_CIPHER_ALGORITHM = 'chacha20';
exports.KEYPAIR_IV_LENGTH = 16;
exports.KEYPAIR_CIPHER_KEY_LENGTH = 32;
exports.KEYPAIR_SALT_LENGTH = 32;
exports.KEYPAIR_SCRYPT_PARAMS = { N: 8192, r: 8, p: 16 };
exports.KEYPAIR_PADDING = Buffer.alloc(16, 0x00);
exports.KEYPAIR_MESSAGE_DIGEST_ALGORITHM = 'sha256';
//# sourceMappingURL=index.js.map