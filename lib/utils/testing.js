"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const crypto_1 = require("crypto");
const levelup_1 = __importDefault(require("levelup"));
const memdown_1 = __importDefault(require("memdown"));
const _1 = require(".");
const __1 = __importDefault(require(".."));
const types_1 = require("../types");
async function getPrivKeyData(_privKey) {
    const privKey = new types_1.Hash(_privKey);
    const pubKey = new types_1.Hash(await _1.privKeyToPubKey(privKey));
    const address = new types_1.AccountId(await _1.pubKeyToAccountId(pubKey));
    return {
        privKey,
        pubKey,
        address,
    };
}
exports.getPrivKeyData = getPrivKeyData;
async function generateUser(web3, funder, hoprToken) {
    const user = await getPrivKeyData(crypto_1.randomBytes(32));
    // fund user with ETH
    await web3.eth.sendTransaction({
        value: web3.utils.toWei('1', 'ether'),
        from: funder.address.toHex(),
        to: user.address.toHex(),
    });
    // mint user some HOPR
    await hoprToken.methods.mint(user.address.toHex(), web3.utils.toWei('1', 'ether'), '0x00', '0x00').send({
        from: funder.address.toHex(),
        gas: 200e3,
    });
    return user;
}
exports.generateUser = generateUser;
async function generateNode(privKey) {
    return __1.default.create(new levelup_1.default(memdown_1.default()), privKey);
}
exports.generateNode = generateNode;
