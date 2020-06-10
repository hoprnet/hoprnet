"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const types_1 = require("./types");
const utils_1 = require("./utils");
const hopr_utils_1 = require("@hoprnet/hopr-utils");
class Account {
    constructor(coreConnector, privKey, pubKey) {
        this.coreConnector = coreConnector;
        this.keys = {
            onChain: {
                privKey,
                pubKey,
            },
            offChain: {
                privKey,
                pubKey,
            },
        };
    }
    get nonce() {
        return new Promise(async (resolve, reject) => {
            try {
                let nonce;
                // 'first' call
                if (typeof this._nonce === 'undefined') {
                    this._nonce = {
                        getTransactionCount: this.coreConnector.web3.eth.getTransactionCount((await this.address).toHex()),
                        virtualNonce: 0,
                        nonce: undefined,
                    };
                    nonce = await this._nonce.getTransactionCount;
                }
                // called while 'first' call hasn't returned
                else if (typeof this._nonce.nonce === 'undefined') {
                    this._nonce.virtualNonce += 1;
                    const virtualNonce = this._nonce.virtualNonce;
                    nonce = await this._nonce.getTransactionCount.then((count) => {
                        return count + virtualNonce;
                    });
                }
                // called after 'first' call has returned
                else {
                    nonce = this._nonce.nonce + 1;
                }
                this._nonce.nonce = nonce;
                return resolve(nonce);
            }
            catch (err) {
                return reject(err.message);
            }
        });
    }
    get balance() {
        return new Promise(async (resolve, reject) => {
            try {
                resolve(new types_1.Balance(await this.coreConnector.hoprToken.methods.balanceOf((await this.address).toHex()).call()));
            }
            catch (err) {
                reject(err);
            }
        });
    }
    get nativeBalance() {
        return new Promise(async (resolve, reject) => {
            try {
                resolve(new types_1.NativeBalance(await this.coreConnector.web3.eth.getBalance((await this.address).toHex())));
            }
            catch (err) {
                reject(err);
            }
        });
    }
    get ticketEpoch() {
        return new Promise(async (resolve, reject) => {
            try {
                resolve(new types_1.TicketEpoch((await this.coreConnector.hoprChannels.methods.accounts((await this.address).toHex()).call()).counter));
            }
            catch (err) {
                reject(err);
            }
        });
    }
    /**
     * Returns the current value of the onChainSecret
     */
    get onChainSecret() {
        return new Promise(async (resolve, reject) => {
            try {
                resolve(new types_1.Hash(hopr_utils_1.stringToU8a((await this.coreConnector.hoprChannels.methods.accounts((await this.address).toHex()).call()).hashedSecret)));
            }
            catch (err) {
                reject(err);
            }
        });
    }
    get address() {
        return utils_1.pubKeyToAccountId(this.keys.onChain.pubKey);
    }
}
exports.default = Account;
//# sourceMappingURL=account.js.map