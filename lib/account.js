"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const hopr_utils_1 = require("@hoprnet/hopr-utils");
const types_1 = require("./types");
const utils_1 = require("./utils");
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
        this._nonceIterator = async function* () {
            let nonce = await this.coreConnector.web3.eth.getTransactionCount((await this.address).toHex());
            while (true) {
                yield nonce++;
            }
        }.call(this);
    }
    get nonce() {
        return this._nonceIterator.next().then((res) => res.value);
    }
    /**
     * Returns the current balances of the account associated with this node (HOPR)
     * @returns a promise resolved to Balance
     */
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
    /**
     * Returns the current native balance (ETH)
     * @returns a promise resolved to Balance
     */
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
                if (typeof this._ticketEpoch !== 'undefined') {
                    return resolve(this._ticketEpoch);
                }
                // listen for 'SecretHashSet' events and update 'ticketEpoch'
                // on error, safely reset 'ticketEpoch' & event listener
                this._ticketEpochListener = this.coreConnector.hoprChannels.events
                    .SecretHashSet({
                    fromBlock: 'latest',
                    filter: {
                        account: (await this.address).toHex(),
                    },
                })
                    .on('data', (event) => {
                    this.coreConnector.log('new ticketEpoch', event.returnValues.counter);
                    this._ticketEpoch = new types_1.TicketEpoch(event.returnValues.counter);
                })
                    .on('error', (error) => {
                    this.coreConnector.log('error listening to SecretHashSet events', error.message);
                    this._ticketEpochListener.removeAllListeners();
                    this._ticketEpoch = undefined;
                });
                this._ticketEpoch = new types_1.TicketEpoch((await this.coreConnector.hoprChannels.methods.accounts((await this.address).toHex()).call()).counter);
                resolve(this._ticketEpoch);
            }
            catch (err) {
                // reset everything on unexpected error
                this._ticketEpochListener.removeAllListeners();
                this._ticketEpoch = undefined;
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
        if (this._address) {
            return Promise.resolve(this._address);
        }
        return utils_1.pubKeyToAccountId(this.keys.onChain.pubKey).then((accountId) => {
            this._address = accountId;
            return this._address;
        });
    }
}
exports.default = Account;
//# sourceMappingURL=account.js.map