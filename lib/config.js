"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.MAX_CONFIRMATIONS = exports.DEMO_ACCOUNTS = exports.FUND_ACCOUNT_PRIVATE_KEY = exports.CHANNELS_ADDRESSES = exports.TOKEN_ADDRESSES = exports.DEFAULT_URI = void 0;
const hopr_demo_seeds_1 = require("@hoprnet/hopr-demo-seeds");
const hopr_ethereum_1 = require("@hoprnet/hopr-ethereum");
exports.DEFAULT_URI = 'ws://127.0.0.1:9545/';
exports.TOKEN_ADDRESSES = hopr_ethereum_1.addresses.HOPR_TOKEN;
exports.CHANNELS_ADDRESSES = hopr_ethereum_1.addresses.HOPR_CHANNELS;
exports.FUND_ACCOUNT_PRIVATE_KEY = hopr_demo_seeds_1.NODE_SEEDS[0];
exports.DEMO_ACCOUNTS = hopr_demo_seeds_1.NODE_SEEDS;
exports.MAX_CONFIRMATIONS = 8;
//# sourceMappingURL=config.js.map