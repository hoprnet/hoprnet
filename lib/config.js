"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.MAX_CONFIRMATIONS = exports.CHANNELS_ADDRESSES = exports.TOKEN_ADDRESSES = exports.DEFAULT_URI = void 0;
const hopr_ethereum_1 = require("@hoprnet/hopr-ethereum");
exports.DEFAULT_URI = 'ws://127.0.0.1:9545/';
exports.TOKEN_ADDRESSES = hopr_ethereum_1.addresses.HOPR_TOKEN;
exports.CHANNELS_ADDRESSES = hopr_ethereum_1.addresses.HOPR_CHANNELS;
exports.MAX_CONFIRMATIONS = 8;
//# sourceMappingURL=config.js.map