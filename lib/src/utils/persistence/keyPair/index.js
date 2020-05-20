"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    Object.defineProperty(o, k2, { enumerable: true, get: function() { return m[k]; } });
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __exportStar = (this && this.__exportStar) || function(m, exports) {
    for (var p in m) if (p !== "default" && !exports.hasOwnProperty(p)) __createBinding(exports, m, p);
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.KEYPAIR_MESSAGE_DIGEST_ALGORITHM = exports.KEYPAIR_PADDING = exports.KEYPAIR_SCRYPT_PARAMS = exports.KEYPAIR_SALT_LENGTH = exports.KEYPAIR_CIPHER_KEY_LENGTH = exports.KEYPAIR_IV_LENGTH = exports.KEYPAIR_CIPHER_ALGORITHM = void 0;
__exportStar(require("./deserialize"), exports);
__exportStar(require("./serialize"), exports);
exports.KEYPAIR_CIPHER_ALGORITHM = 'chacha20';
exports.KEYPAIR_IV_LENGTH = 16;
exports.KEYPAIR_CIPHER_KEY_LENGTH = 32;
exports.KEYPAIR_SALT_LENGTH = 32;
exports.KEYPAIR_SCRYPT_PARAMS = { N: 8192, r: 8, p: 16 };
exports.KEYPAIR_PADDING = Buffer.alloc(16, 0x00);
exports.KEYPAIR_MESSAGE_DIGEST_ALGORITHM = 'sha256';
//# sourceMappingURL=index.js.map