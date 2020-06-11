"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.disconnectWeb3 = exports.createNode = exports.createAccountAndFund = exports.createAccount = exports.fundAccount = exports.getPrivKeyData = void 0;
const crypto_1 = require("crypto");
const levelup_1 = __importDefault(require("levelup"));
const memdown_1 = __importDefault(require("memdown"));
const hopr_utils_1 = require("@hoprnet/hopr-utils");
const _1 = require(".");
const __1 = __importDefault(require(".."));
const types_1 = require("../types");
/**
 * Return private key data like public key and address.
 * @param _privKey private key to derive data from
 */
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
/**
 * Fund an account.
 * @param web3 the web3 instance the our hoprToken contract is deployed to
 * @param hoprToken the hoprToken instance that will be used to mint tokens
 * @param funder object
 * @param account object
 */
async function fundAccount(web3, hoprToken, funder, account) {
    // fund account with ETH
    await web3.eth.sendTransaction({
        value: web3.utils.toWei('1', 'ether'),
        from: funder.address.toHex(),
        to: account.address.toHex(),
    });
    // mint account some HOPR
    await hoprToken.methods.mint(account.address.toHex(), web3.utils.toWei('1', 'ether'), '0x00', '0x00').send({
        from: funder.address.toHex(),
        gas: 200e3,
    });
}
exports.fundAccount = fundAccount;
/**
 * Create a random account.
 * @param privKey the private key of the connector
 * @returns CoreConnector
 */
async function createAccount() {
    return getPrivKeyData(crypto_1.randomBytes(types_1.Hash.SIZE));
}
exports.createAccount = createAccount;
/**
 * Create a random account or use provided one, and then fund it.
 * @param privKey the private key of the connector
 * @returns CoreConnector
 */
async function createAccountAndFund(web3, hoprToken, funder, account) {
    if (typeof account === 'undefined') {
        account = await createAccount();
    }
    else if (typeof account === 'string') {
        account = await getPrivKeyData(hopr_utils_1.stringToU8a(account));
    }
    else if (account instanceof Uint8Array) {
        account = await getPrivKeyData(account);
    }
    await fundAccount(web3, hoprToken, funder, account);
    return account;
}
exports.createAccountAndFund = createAccountAndFund;
/**
 * Given a private key, create a connector node.
 * @param privKey the private key of the connector
 * @returns CoreConnector
 */
async function createNode(privKey) {
    return __1.default.create(new levelup_1.default(memdown_1.default()), privKey, {
        debug: true,
    });
}
exports.createNode = createNode;
/**
 * Disconnect web3 as if it lost connection
 * @param web3 Web3 instance
 */
async function disconnectWeb3(web3) {
    try {
        // @ts-ignore
        return web3.currentProvider.disconnect(4000);
    }
    catch (err) {
        // console.error(err)
    }
}
exports.disconnectWeb3 = disconnectWeb3;
//# sourceMappingURL=testing.js.map