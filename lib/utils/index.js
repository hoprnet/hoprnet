"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (Object.hasOwnProperty.call(mod, k)) result[k] = mod[k];
    result["default"] = mod;
    return result;
};
Object.defineProperty(exports, "__esModule", { value: true });
const assert_1 = __importDefault(require("assert"));
const secp256k1_1 = require("secp256k1");
const keccak_1 = __importDefault(require("keccak"));
const web3_1 = __importDefault(require("web3"));
const debug_1 = __importDefault(require("debug"));
const types_1 = require("../types");
const channel_1 = require("../types/channel");
const constants = __importStar(require("../constants"));
function isPartyA(self, counterparty) {
    return Buffer.compare(self, counterparty) < 0;
}
exports.isPartyA = isPartyA;
function getParties(self, counterparty) {
    if (isPartyA(self, counterparty)) {
        return [self, counterparty];
    }
    else {
        return [counterparty, self];
    }
}
exports.getParties = getParties;
function getId(self, counterparty) {
    return hash(Buffer.concat(getParties(self, counterparty), 2 * constants.ADDRESS_LENGTH));
}
exports.getId = getId;
async function privKeyToPubKey(privKey) {
    if (privKey.length != constants.PRIVATE_KEY_LENGTH)
        throw Error(`Invalid input parameter. Expected a Buffer of size ${constants.PRIVATE_KEY_LENGTH}. Got '${typeof privKey}'${privKey.length ? ` of length ${privKey.length}` : ''}.`);
    return secp256k1_1.publicKeyCreate(privKey);
}
exports.privKeyToPubKey = privKeyToPubKey;
async function pubKeyToAccountId(pubKey) {
    if (pubKey.length != constants.COMPRESSED_PUBLIC_KEY_LENGTH)
        throw Error(`Invalid input parameter. Expected a Buffer of size ${constants.COMPRESSED_PUBLIC_KEY_LENGTH}. Got '${typeof pubKey}'${pubKey.length ? ` of length ${pubKey.length}` : ''}.`);
    return new types_1.AccountId((await hash(secp256k1_1.publicKeyConvert(pubKey, false).slice(1))).slice(12));
}
exports.pubKeyToAccountId = pubKeyToAccountId;
async function hash(msg) {
    return Promise.resolve(new types_1.Hash(keccak_1.default('keccak256').update(Buffer.from(msg)).digest()));
}
exports.hash = hash;
async function sign(msg, privKey, pubKey, arr) {
    const result = secp256k1_1.ecdsaSign(msg, privKey);
    const response = new types_1.Signature(arr, {
        signature: result.signature,
        // @ts-ignore-next-line
        recovery: result.recid
    });
    return response;
}
exports.sign = sign;
async function signer(msg, signature) {
    return secp256k1_1.ecdsaRecover(signature.signature, signature.recovery, msg);
}
exports.signer = signer;
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
async function waitForConfirmation(event) {
    return new Promise((resolve, reject) => {
        return event
            .on('receipt', receipt => {
            resolve(receipt);
        })
            .on("error", err => {
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
function advanceBlockAtTime(web3, time) {
    return new Promise((resolve, reject) => {
        // @ts-ignore
        web3.currentProvider.send({
            jsonrpc: '2.0',
            method: 'evm_mine',
            params: [time],
            id: new Date().getTime()
        }, async (err) => {
            if (err) {
                return reject(err);
            }
            const newBlock = await web3.eth.getBlock('latest');
            const newBlockHash = newBlock.hash;
            return resolve(newBlockHash);
        });
    });
}
exports.advanceBlockAtTime = advanceBlockAtTime;
async function wait(ms) {
    return new Promise(resolve => {
        setTimeout(resolve, ms);
    });
}
exports.wait = wait;
async function waitFor({ web3, network, getCurrentBlock, timestamp }) {
    const now = await getCurrentBlock().then(block => Number(block.timestamp) * 1e3);
    if (timestamp < now) {
        return undefined;
    }
    if (network === 'private') {
        await advanceBlockAtTime(web3, Math.ceil(timestamp / 1e3) + 1);
    }
    else {
        const diff = now - timestamp || 60 * 1e3;
        await wait(diff);
    }
    return waitFor({
        web3,
        network,
        getCurrentBlock,
        timestamp: await getCurrentBlock().then(block => Number(block.timestamp) * 1e3)
    });
}
exports.waitFor = waitFor;
/*
  return network name, not using web3 'getNetworkType' because
  it misses networks & uses genesis block to determine networkid.
  supports all infura networks
*/
async function getNetworkId(web3) {
    return web3.eth.net.getId().then(netId => {
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
function stateCountToStatus(stateCount) {
    const status = Number(stateCount) % 10;
    if (status >= Object.keys(channel_1.ChannelStatus).length) {
        throw Error("status like this doesn't exist");
    }
    return status;
}
exports.stateCountToStatus = stateCountToStatus;
// sign transaction's locally and send them
// @TODO: switch to web3js-accounts wallet if it's safe
// @TODO: remove explicit any
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
            data: abi
        }, privKeyStr);
        function send() {
            return web3.eth.sendSignedTransaction(signedTransaction.rawTransaction);
        }
        return {
            send,
            transactionHash: signedTransaction.transactionHash
        };
    };
}
exports.TransactionSigner = TransactionSigner;
function Log(suffixes = []) {
    return debug_1.default(["hopr-core-ethereum"].concat(suffixes).join(":"));
}
exports.Log = Log;
