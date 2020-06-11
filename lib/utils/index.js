"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    Object.defineProperty(o, k2, { enumerable: true, get: function() { return m[k]; } });
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.getSignatureParameters = exports.cleanupPromiEvent = exports.Log = exports.TransactionSigner = exports.stateCountToStatus = exports.getNetworkId = exports.waitFor = exports.wait = exports.waitForConfirmation = exports.convertUnit = exports.verify = exports.signer = exports.sign = exports.hash = exports.pubKeyToAccountId = exports.privKeyToPubKey = exports.getId = exports.getParties = exports.isPartyA = exports.time = void 0;
const assert_1 = __importDefault(require("assert"));
const secp256k1_1 = require("secp256k1");
const keccak_1 = __importDefault(require("keccak"));
const web3_1 = __importDefault(require("web3"));
const debug_1 = __importDefault(require("debug"));
const types_1 = require("../types");
const channel_1 = require("../types/channel");
const constants = __importStar(require("../constants"));
const time = __importStar(require("./time"));
exports.time = time;
/**
 * @param self our node's accountId
 * @param counterparty counterparty's accountId
 * @returns true if self is partyA
 */
function isPartyA(self, counterparty) {
    return Buffer.compare(self, counterparty) < 0;
}
exports.isPartyA = isPartyA;
/**
 * @param self our node's accountId
 * @param counterparty counterparty's accountId
 * @returns an array of partyA's and partyB's accountIds
 */
function getParties(self, counterparty) {
    if (isPartyA(self, counterparty)) {
        return [self, counterparty];
    }
    else {
        return [counterparty, self];
    }
}
exports.getParties = getParties;
/**
 * Get the channel id of self and counterparty
 * @param self our node's accountId
 * @param counterparty counterparty's accountId
 * @returns a promise resolved to Hash
 */
function getId(self, counterparty) {
    return hash(Buffer.concat(getParties(self, counterparty), 2 * constants.ADDRESS_LENGTH));
}
exports.getId = getId;
/**
 * Given a private key, derive public key.
 * @param privKey the private key to derive the public key from
 * @returns a promise resolved to Uint8Array
 */
async function privKeyToPubKey(privKey) {
    if (privKey.length != constants.PRIVATE_KEY_LENGTH)
        throw Error(`Invalid input parameter. Expected a Buffer of size ${constants.PRIVATE_KEY_LENGTH}. Got '${typeof privKey}'${privKey.length ? ` of length ${privKey.length}` : ''}.`);
    return secp256k1_1.publicKeyCreate(privKey);
}
exports.privKeyToPubKey = privKeyToPubKey;
/**
 * Given a public key, derive the AccountId.
 * @param pubKey the public key to derive the AccountId from
 * @returns a promise resolved to AccountId
 */
async function pubKeyToAccountId(pubKey) {
    if (pubKey.length != constants.COMPRESSED_PUBLIC_KEY_LENGTH)
        throw Error(`Invalid input parameter. Expected a Buffer of size ${constants.COMPRESSED_PUBLIC_KEY_LENGTH}. Got '${typeof pubKey}'${pubKey.length ? ` of length ${pubKey.length}` : ''}.`);
    return new types_1.AccountId((await hash(secp256k1_1.publicKeyConvert(pubKey, false).slice(1))).slice(12));
}
exports.pubKeyToAccountId = pubKeyToAccountId;
/**
 * Given a message, generate hash using keccak256.
 * @param msg the message to hash
 * @returns a promise resolved to Hash
 */
async function hash(msg) {
    return Promise.resolve(new types_1.Hash(keccak_1.default('keccak256').update(Buffer.from(msg)).digest()));
}
exports.hash = hash;
/**
 * Sign message.
 * @param msg the message to sign
 * @param privKey the private key to use when signing
 * @param pubKey deprecated
 * @param arr
 * @returns a promise resolved to Hash
 */
async function sign(msg, privKey, pubKey, arr) {
    const result = secp256k1_1.ecdsaSign(msg, privKey);
    const response = new types_1.Signature(arr, {
        signature: result.signature,
        // @ts-ignore-next-line
        recovery: result.recid,
    });
    return response;
}
exports.sign = sign;
/**
 * Recover signer.
 * @param msg the message that was signed
 * @param signature the signature of the signed message
 * @returns a promise resolved to Uint8Array, the signers public key
 */
async function signer(msg, signature) {
    return secp256k1_1.ecdsaRecover(signature.signature, signature.recovery, msg);
}
exports.signer = signer;
/**
 * Verify signer.
 * @param msg the message that was signed
 * @param signature the signature of the signed message
 * @param pubKey the public key of the potential signer
 * @returns a promise resolved to true if the public key provided matches the signer's
 */
async function verify(msg, signature, pubKey) {
    return secp256k1_1.ecdsaVerify(signature.signature, msg, pubKey);
}
exports.verify = verify;
function convertUnit(amount, sourceUnit, targetUnit) {
    assert_1.default(['eth', 'wei'].includes(sourceUnit), 'not implemented');
    if (sourceUnit === 'eth') {
        return web3_1.default.utils.toWei(amount, targetUnit);
    }
    else {
        return web3_1.default.utils.fromWei(amount, targetUnit);
    }
}
exports.convertUnit = convertUnit;
/**
 * Wait until block has been confirmed.
 *
 * @typeparam T Our PromiEvent
 * @param event Our event, returned by web3
 * @returns the transaction receipt
 */
async function waitForConfirmation(event) {
    return new Promise((resolve, reject) => {
        return event
            .on('receipt', (receipt) => {
            resolve(receipt);
        })
            .on('error', (err) => {
            const outOfEth = err.message.includes(`enough funds`);
            const outOfHopr = err.message.includes(`SafeERC20:`);
            if (outOfEth) {
                return reject(Error(constants.ERRORS.OOF_ETH));
            }
            else if (outOfHopr) {
                return reject(Error(constants.ERRORS.OOF_HOPR));
            }
            else {
                return reject(err);
            }
        });
    });
}
exports.waitForConfirmation = waitForConfirmation;
/**
 * An asychronous setTimeout.
 *
 * @param ms milliseconds to wait
 */
async function wait(ms) {
    return new Promise((resolve) => {
        setTimeout(resolve, ms);
    });
}
exports.wait = wait;
/**
 * Wait until timestamp is reached onchain.
 *
 * @param ms milliseconds to wait
 */
async function waitFor({ web3, network, getCurrentBlock, timestamp, }) {
    const now = await getCurrentBlock().then((block) => Number(block.timestamp) * 1e3);
    if (timestamp < now) {
        return undefined;
    }
    const diff = now - timestamp || 60;
    if (network === 'private') {
        await time.increase(web3, diff);
    }
    else {
        await wait(diff * 1e3);
    }
    return waitFor({
        web3,
        network,
        getCurrentBlock,
        timestamp: await getCurrentBlock().then((block) => Number(block.timestamp) * 1e3),
    });
}
exports.waitFor = waitFor;
/**
 * Get current network's name.
 *
 * @param web3 a web3 instance
 * @returns the network's name
 */
async function getNetworkId(web3) {
    return web3.eth.net.getId().then((netId) => {
        switch (netId) {
            case 1:
                return 'mainnet';
            case 2:
                return 'morden';
            case 3:
                return 'ropsten';
            case 4:
                return 'rinkeby';
            case 5:
                return 'goerli';
            case 42:
                return 'kovan';
            default:
                return 'private';
        }
    });
}
exports.getNetworkId = getNetworkId;
/**
 * Convert a state count (one received from on-chain),
 * to an enumarated representation.
 *
 * @param stateCount the state count
 * @returns ChannelStatus
 */
function stateCountToStatus(stateCount) {
    const status = Number(stateCount) % 10;
    if (status >= Object.keys(channel_1.ChannelStatus).length) {
        throw Error("status like this doesn't exist");
    }
    return status;
}
exports.stateCountToStatus = stateCountToStatus;
/**
 * A signer factory that signs transactions using the given private key.
 *
 * @param web3 a web3 instance
 * @param privKey the private key to sign transactions with
 * @returns signer
 */
// @TODO: switch to web3js-accounts wallet if it's safe
function TransactionSigner(web3, privKey) {
    const privKeyStr = new types_1.Hash(privKey).toHex();
    return async function signTransaction(
    // return of our contract method in web3.Contract instance
    txObject, 
    // config put in .send
    txConfig) {
        const abi = txObject.encodeABI();
        // estimation is not always right, adding some more
        // const estimatedGas = Math.floor((await txObject.estimateGas()) * 1.25)
        const estimatedGas = 200e3;
        // @TODO: provide some of the values to avoid multiple calls
        const signedTransaction = await web3.eth.accounts.signTransaction({
            gas: estimatedGas,
            ...txConfig,
            data: abi,
        }, privKeyStr);
        function send() {
            return web3.eth.sendSignedTransaction(signedTransaction.rawTransaction);
        }
        return {
            send,
            transactionHash: signedTransaction.transactionHash,
        };
    };
}
exports.TransactionSigner = TransactionSigner;
/**
 * Create a prefixed Debug instance.
 *
 * @param prefixes an array containing prefixes
 * @returns a debug instance prefixed by joining 'prefixes'
 */
function Log(prefixes = []) {
    return debug_1.default(['hopr-core-ethereum'].concat(prefixes).join(':'));
}
exports.Log = Log;
/**
 * Once function 'fn' resolves, remove all listeners from 'event'.
 *
 * @typeparam E Our contract event emitteer
 * @typeparam R fn's return
 * @param event an event
 * @param fn a function to wait for
 */
async function cleanupPromiEvent(event, fn) {
    return fn(event).finally(() => event.removeAllListeners());
}
exports.cleanupPromiEvent = cleanupPromiEvent;
/**
 * Get r,s,v values of a signature
 */
function getSignatureParameters(signature) {
    return {
        r: signature.signature.slice(0, 32),
        s: signature.signature.slice(32, 64),
        v: signature.recovery,
    };
}
exports.getSignatureParameters = getSignatureParameters;
//# sourceMappingURL=index.js.map