"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const hopr_utils_1 = require("@hoprnet/hopr-utils");
const types_1 = require("../types");
const ticket_1 = __importDefault(require("./ticket"));
const channel_1 = require("../types/channel");
const utils_1 = require("../utils");
class Channel {
    constructor(coreConnector, counterparty, signedChannel) {
        this.coreConnector = coreConnector;
        this.counterparty = counterparty;
        this._signedChannel = signedChannel;
        // check if channel still exists
        this.status.then((status) => {
            if (status === channel_1.ChannelStatus.UNINITIALISED) {
                this.coreConnector.log.log('found channel off-chain but its closed on-chain');
                this.onClose();
            }
        });
        // if channel is closed
        this.onceClosed().then(async () => {
            return this.onClose();
        });
        this.ticket = new ticket_1.default(this);
    }
    async onceClosed() {
        return this.coreConnector.channel.onceClosed(await this.coreConnector.account.address, await this.coreConnector.utils.pubKeyToAccountId(this.counterparty));
    }
    // private async onOpen(): Promise<void> {
    //   return onOpen(this.coreConnector, this.counterparty, this._signedChannel)
    // }
    async onClose() {
        return this.coreConnector.channel.deleteOffChainState(this.counterparty);
    }
    get channel() {
        return new Promise(async (resolve, reject) => {
            try {
                return resolve(await this.coreConnector.channel.getOnChainState(await this.channelId));
            }
            catch (error) {
                return reject(error);
            }
        });
    }
    get status() {
        return new Promise(async (resolve, reject) => {
            try {
                const channel = await this.channel;
                const status = Number(channel.stateCounter) % 10;
                if (status >= Object.keys(channel_1.ChannelStatus).length) {
                    throw Error("status like this doesn't exist");
                }
                return resolve(status);
            }
            catch (error) {
                return reject(error);
            }
        });
    }
    get offChainCounterparty() {
        return Promise.resolve(this.counterparty);
    }
    get channelId() {
        if (this._channelId != null) {
            return Promise.resolve(this._channelId);
        }
        return new Promise(async (resolve, reject) => {
            try {
                this._channelId = new types_1.ChannelId(await this.coreConnector.utils.getId(await this.coreConnector.account.address, await this.coreConnector.utils.pubKeyToAccountId(this.counterparty)));
            }
            catch (error) {
                return reject(error);
            }
            return resolve(this._channelId);
        });
    }
    get settlementWindow() {
        if (this._settlementWindow != null) {
            return Promise.resolve(this._settlementWindow);
        }
        return new Promise(async (resolve, reject) => {
            try {
                this._settlementWindow = new types_1.Moment((await this.channel).closureTime);
            }
            catch (error) {
                return reject(error);
            }
            return resolve(this._settlementWindow);
        });
    }
    get state() {
        return Promise.resolve(this._signedChannel.channel);
    }
    get balance() {
        return new Promise(async (resolve, reject) => {
            try {
                return resolve(new types_1.Balance((await this.channel).deposit));
            }
            catch (error) {
                return reject(error);
            }
        });
    }
    get balance_a() {
        return new Promise(async (resolve, reject) => {
            try {
                return resolve(new types_1.Balance((await this.channel).partyABalance));
            }
            catch (error) {
                return reject(error);
            }
        });
    }
    get currentBalance() {
        return new Promise(async (resolve, reject) => {
            try {
                return resolve(new types_1.Balance(await this.coreConnector.hoprToken.methods
                    .balanceOf(hopr_utils_1.u8aToHex(await this.coreConnector.account.address))
                    .call()));
            }
            catch (error) {
                return reject(error);
            }
        });
    }
    get currentBalanceOfCounterparty() {
        return new Promise(async (resolve, reject) => {
            try {
                return resolve(new types_1.Balance(await this.coreConnector.hoprToken.methods
                    .balanceOf(hopr_utils_1.u8aToHex(await this.coreConnector.utils.pubKeyToAccountId(this.counterparty)))
                    .call()));
            }
            catch (error) {
                return reject(error);
            }
        });
    }
    async initiateSettlement() {
        // @TODO check out whether we can cache this.channel is some way
        let channel = await this.channel;
        const status = await this.status;
        try {
            if (!(status === channel_1.ChannelStatus.OPEN || status === channel_1.ChannelStatus.PENDING)) {
                throw Error("channel must be 'OPEN' or 'PENDING'");
            }
            if (status === channel_1.ChannelStatus.OPEN) {
                await utils_1.waitForConfirmation((await this.coreConnector.signTransaction(this.coreConnector.hoprChannels.methods.initiateChannelClosure(hopr_utils_1.u8aToHex(await this.coreConnector.utils.pubKeyToAccountId(this.counterparty))), {
                    from: (await this.coreConnector.account.address).toHex(),
                    to: this.coreConnector.hoprChannels.options.address,
                    nonce: await this.coreConnector.account.nonce,
                })).send());
                channel = await this.coreConnector.channel.getOnChainState(await this.channelId);
                await utils_1.waitFor({
                    web3: this.coreConnector.web3,
                    network: this.coreConnector.network,
                    getCurrentBlock: async () => {
                        return this.coreConnector.web3.eth.getBlockNumber().then((blockNumber) => {
                            return this.coreConnector.web3.eth.getBlock(blockNumber);
                        });
                    },
                    timestamp: Number(channel.closureTime) * 1e3,
                });
                await utils_1.waitForConfirmation((await this.coreConnector.signTransaction(this.coreConnector.hoprChannels.methods.claimChannelClosure(hopr_utils_1.u8aToHex(await this.coreConnector.utils.pubKeyToAccountId(this.counterparty))), {
                    from: (await this.coreConnector.account.address).toHex(),
                    to: this.coreConnector.hoprChannels.options.address,
                    nonce: await this.coreConnector.account.nonce,
                })).send());
            }
            else {
                await this.onceClosed();
            }
            await this.onClose();
        }
        catch (error) {
            throw error;
        }
    }
    // @TODO: remove this, no longer needed
    async getPreviousChallenges() {
        return new types_1.Hash();
    }
    async testAndSetNonce(signature) {
        const key = new types_1.Hash(this.coreConnector.dbKeys.Nonce(await this.channelId, await utils_1.hash(signature))).toHex();
        try {
            await this.coreConnector.db.get(key);
        }
        catch (err) {
            if (err.notFound) {
                await this.coreConnector.db.put(key, new Uint8Array());
                return;
            }
            throw err;
        }
        throw Error('Nonces must not be used twice.');
    }
}
exports.default = Channel;
//# sourceMappingURL=channel.js.map