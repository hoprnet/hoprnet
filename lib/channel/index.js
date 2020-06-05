"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const hopr_utils_1 = require("@hoprnet/hopr-utils");
const bn_js_1 = __importDefault(require("bn.js"));
const types_1 = require("../types");
const channel_1 = require("../types/channel");
const constants_1 = require("../constants");
const utils_1 = require("../utils");
const extended_1 = require("../types/extended");
const crypto_1 = require("crypto");
const WIN_PROB = new bn_js_1.default(1);
async function getChannel(coreConnector, channelId) {
    return coreConnector.hoprChannels.methods.channels(channelId.toHex()).call();
}
const onceOpen = async (coreConnector, self, counterparty) => {
    const channelId = await utils_1.getId(self, counterparty);
    return utils_1.cleanupPromiEvent(coreConnector.hoprChannels.events.OpenedChannel({
        filter: {
            opener: [self.toHex(), counterparty.toHex()],
            counterParty: [self.toHex(), counterparty.toHex()],
        },
    }), (event) => {
        return new Promise((resolve, reject) => {
            event
                .on('data', async (data) => {
                const { opener, counterParty } = data.returnValues;
                const _channelId = await coreConnector.utils.getId(new types_1.AccountId(hopr_utils_1.stringToU8a(opener)), new types_1.AccountId(hopr_utils_1.stringToU8a(counterParty)));
                if (!hopr_utils_1.u8aEquals(_channelId, channelId)) {
                    return;
                }
                return resolve(data.returnValues);
            })
                .on('error', reject);
        });
    });
};
const onceClosed = async (coreConnector, self, counterparty) => {
    const channelId = await utils_1.getId(self, counterparty);
    return utils_1.cleanupPromiEvent(coreConnector.hoprChannels.events.ClosedChannel({
        filter: {
            closer: [self.toHex(), counterparty.toHex()],
            counterParty: [self.toHex(), counterparty.toHex()],
        },
    }), (event) => {
        return new Promise((resolve, reject) => {
            event
                .on('data', async (data) => {
                const { closer, counterParty } = data.returnValues;
                const _channelId = await coreConnector.utils.getId(new types_1.AccountId(hopr_utils_1.stringToU8a(closer)), new types_1.AccountId(hopr_utils_1.stringToU8a(counterParty)));
                if (!hopr_utils_1.u8aEquals(_channelId, channelId)) {
                    return;
                }
                resolve(data.returnValues);
            })
                .on('error', reject);
        });
    });
};
const onOpen = async (coreConnector, counterparty, signedChannel) => {
    return coreConnector.db.put(Buffer.from(coreConnector.dbKeys.Channel(counterparty)), Buffer.from(signedChannel));
};
const onClose = async (coreConnector, counterparty) => {
    return coreConnector.db.del(Buffer.from(coreConnector.dbKeys.Channel(counterparty)));
};
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
        this.ticket = {
            create: types_1.Ticket.create.bind(this),
            SIZE: types_1.Ticket.SIZE,
            submit: types_1.Ticket.submit.bind(this),
            verify: types_1.Ticket.verify.bind(this),
        };
    }
    // private async onceOpen() {
    //   return onceOpen(
    //     this.coreConnector,
    //     this.coreConnector.account,
    //     await this.coreConnector.utils.pubKeyToAccountId(this.counterparty)
    //   )
    // }
    async onceClosed() {
        return onceClosed(this.coreConnector, this.coreConnector.account, await this.coreConnector.utils.pubKeyToAccountId(this.counterparty));
    }
    // private async onOpen(): Promise<void> {
    //   return onOpen(this.coreConnector, this.counterparty, this._signedChannel)
    // }
    async onClose() {
        return onClose(this.coreConnector, this.counterparty);
    }
    get channel() {
        return new Promise(async (resolve, reject) => {
            try {
                const response = await getChannel(this.coreConnector, await this.channelId);
                return resolve(response);
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
                this._channelId = new types_1.ChannelId(await this.coreConnector.utils.getId(this.coreConnector.account, await this.coreConnector.utils.pubKeyToAccountId(this.counterparty)));
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
        return new Promise(async (resolve, reject) => {
            try {
                const channel = await this.channel;
                const status = utils_1.stateCountToStatus(Number(channel.stateCounter));
                return resolve(new types_1.State(undefined, {
                    // @TODO: implement this once on-chain channel secrets are added
                    secret: new types_1.Hash(new Uint8Array(types_1.Hash.SIZE).fill(0x0)),
                    // not needed
                    pubkey: new types_1.Public(new Uint8Array(types_1.Public.SIZE).fill(0x0)),
                    epoch: new types_1.TicketEpoch(status),
                }));
            }
            catch (error) {
                return reject(error);
            }
        });
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
                return resolve(new types_1.Balance(await this.coreConnector.hoprToken.methods.balanceOf(hopr_utils_1.u8aToHex(this.coreConnector.account)).call()));
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
                    from: this.coreConnector.account.toHex(),
                    to: this.coreConnector.hoprChannels.options.address,
                    nonce: await this.coreConnector.nonce,
                })).send());
                channel = await getChannel(this.coreConnector, await this.channelId);
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
                    from: this.coreConnector.account.toHex(),
                    to: this.coreConnector.hoprChannels.options.address,
                    nonce: await this.coreConnector.nonce,
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
    async getPreviousChallenges() {
        let pubKeys = [];
        return new Promise(async (resolve, reject) => {
            this.coreConnector.db
                .createReadStream({
                gte: Buffer.from(this.coreConnector.dbKeys.Challenge(await this.channelId, new Uint8Array(constants_1.HASH_LENGTH).fill(0x00))),
                lte: Buffer.from(this.coreConnector.dbKeys.Challenge(await this.channelId, new Uint8Array(constants_1.HASH_LENGTH).fill(0xff))),
            })
                .on('error', (err) => reject(err))
                .on('data', ({ key, ownKeyHalf }) => {
                const challenge = this.coreConnector.dbKeys.ChallengeKeyParse(key)[1];
                // @TODO: replace this by proper EC-arithmetic once it's implemented in `hopr-core`
                pubKeys.push(new Uint8Array(hopr_utils_1.u8aXOR(false, challenge, new Uint8Array(ownKeyHalf))));
            })
                .on('end', () => {
                if (pubKeys.length > 0) {
                    return resolve(new types_1.Hash(hopr_utils_1.u8aXOR(false, ...pubKeys)));
                }
                resolve();
            });
        });
    }
    async testAndSetNonce(signature) {
        const channelId = await this.channelId;
        const nonce = await utils_1.hash(signature);
        const key = new types_1.Hash(this.coreConnector.dbKeys.Nonce(channelId, nonce)).toHex();
        try {
            await this.coreConnector.db.get(key);
        }
        catch (err) {
            if (err.notFound == null || err.notFound != true) {
                throw err;
            }
            await this.coreConnector.db.put(key, new Uint8Array());
            return;
        }
        throw Error('Nonces must not be used twice.');
    }
    static async isOpen(counterpartyPubKey) {
        const counterparty = await this.utils.pubKeyToAccountId(counterpartyPubKey);
        const channelId = await this.utils.getId(this.account, counterparty).then((res) => new types_1.Hash(res));
        const [onChain, offChain] = await Promise.all([
            getChannel(this, channelId).then((channel) => {
                const state = Number(channel.stateCounter) % 10;
                return state === channel_1.ChannelStatus.OPEN || state === channel_1.ChannelStatus.PENDING;
            }),
            this.db.get(Buffer.from(this.dbKeys.Channel(counterpartyPubKey))).then(() => true, (err) => {
                if (err.notFound) {
                    return false;
                }
                else {
                    throw err;
                }
            }),
        ]);
        if (onChain != offChain) {
            if (!onChain && offChain) {
                this.log(`Channel ${hopr_utils_1.u8aToHex(channelId)} exists off-chain but not on-chain, deleting data.`);
                await onClose(this, counterpartyPubKey);
            }
            else {
                throw Error(`Channel ${hopr_utils_1.u8aToHex(channelId)} exists on-chain but not off-chain.`);
            }
        }
        return onChain && offChain;
    }
    static async increaseFunds(counterparty, amount) {
        try {
            if ((await this.accountBalance).lt(amount)) {
                throw Error(constants_1.ERRORS.OOF_HOPR);
            }
            await utils_1.waitForConfirmation((await this.signTransaction(this.hoprToken.methods.send(this.hoprChannels.options.address, amount.toString(), this.web3.eth.abi.encodeParameters(['address', 'address'], [this.account.toHex(), counterparty.toHex()])), {
                from: this.account.toHex(),
                to: this.hoprToken.options.address,
                nonce: await this.nonce,
            })).send());
        }
        catch (error) {
            throw error;
        }
    }
    static async createDummyChannelTicket(counterParty, challenge, arr) {
        const channelId = await this.utils.getId(await this.utils.pubKeyToAccountId(this.self.onChainKeyPair.publicKey), counterParty);
        // const account = await this.coreConnector.utils.pubKeyToAccountId(counterparty)
        // const { hashedSecret } = await this.coreConnector.hoprChannels.methods.accounts(u8aToHex(account)).call()
        const winProb = new extended_1.Uint8ArrayE(new bn_js_1.default(new Uint8Array(types_1.Hash.SIZE).fill(0xff)).div(WIN_PROB).toArray('le', types_1.Hash.SIZE));
        // const channelId = await this.channelId
        const signedTicket = new types_1.SignedTicket(arr);
        const ticket = new types_1.Ticket({
            bytes: signedTicket.buffer,
            offset: signedTicket.ticketOffset,
        }, {
            channelId,
            challenge,
            // @TODO set this dynamically
            epoch: new types_1.TicketEpoch(0),
            amount: new types_1.Balance(0),
            winProb,
            onChainSecret: new extended_1.Uint8ArrayE(crypto_1.randomBytes(types_1.Hash.SIZE)),
        });
        await this.utils.sign(await ticket.hash, this.self.privateKey, undefined, {
            bytes: signedTicket.buffer,
            offset: signedTicket.signatureOffset,
        });
        return signedTicket;
    }
    static async create(counterpartyPubKey, _getOnChainPublicKey, channelBalance, sign) {
        const counterparty = await this.utils.pubKeyToAccountId(counterpartyPubKey);
        let channel;
        let signedChannel;
        if (!this._onChainValuesInitialized) {
            await this.initOnchainValues();
        }
        if (await this.channel.isOpen(counterpartyPubKey)) {
            const record = await this.db.get(Buffer.from(this.dbKeys.Channel(counterpartyPubKey)));
            signedChannel = new types_1.SignedChannel({
                bytes: record.buffer,
                offset: record.byteOffset,
            });
            channel = new Channel(this, counterpartyPubKey, signedChannel);
        }
        else if (sign != null && channelBalance != null) {
            let amount;
            if (this.utils.isPartyA(this.account, counterparty)) {
                amount = channelBalance.balance_a;
            }
            else {
                amount = new types_1.Balance(channelBalance.balance.sub(channelBalance.balance_a));
            }
            await this.channel.increaseFunds(counterparty, amount);
            signedChannel = await sign(channelBalance);
            channel = new Channel(this, counterpartyPubKey, signedChannel);
            await utils_1.waitForConfirmation((await this.signTransaction(this.hoprChannels.methods.openChannel(counterparty.toHex()), {
                from: this.account.toHex(),
                to: this.hoprChannels.options.address,
                nonce: await this.nonce,
            })).send());
            await this.db.put(Buffer.from(this.dbKeys.Channel(counterpartyPubKey)), Buffer.from(signedChannel));
        }
        else {
            throw Error('Invalid input parameters.');
        }
        return channel;
    }
    static getAll(onData, onEnd) {
        const promises = [];
        return new Promise((resolve, reject) => {
            this.db
                .createReadStream({
                gte: Buffer.from(this.dbKeys.Channel(new Uint8Array(types_1.Hash.SIZE).fill(0x00))),
                lte: Buffer.from(this.dbKeys.Channel(new Uint8Array(types_1.Hash.SIZE).fill(0xff))),
            })
                .on('error', (err) => reject(err))
                .on('data', ({ key, value }) => {
                const signedChannel = new types_1.SignedChannel({
                    bytes: value.buffer,
                    offset: value.byteOffset,
                });
                promises.push(onData(new Channel(this, this.dbKeys.ChannelKeyParse(key), signedChannel)));
            })
                .on('end', () => resolve(onEnd(promises)));
        });
    }
    static async closeChannels() {
        const result = new bn_js_1.default(0);
        return this.channel.getAll((channel) => channel.initiateSettlement().then(() => {
            // @TODO: add balance
            result.iaddn(0);
        }), async (promises) => {
            await Promise.all(promises);
            return new types_1.Balance(result);
        });
    }
    static handleOpeningRequest() {
        return (source) => async function* () {
            for await (const _msg of source) {
                const msg = _msg.slice();
                const signedChannel = new types_1.SignedChannel({
                    bytes: msg.buffer,
                    offset: msg.byteOffset,
                });
                const counterpartyPubKey = await signedChannel.signer;
                const counterparty = new types_1.AccountId(await this.utils.pubKeyToAccountId(counterpartyPubKey));
                const channelBalance = signedChannel.channel.balance;
                if (this.utils.isPartyA(this.account, counterparty)) {
                    if (channelBalance.balance.sub(channelBalance.balance_a).gtn(0)) {
                        await this.channel.increaseFunds(counterparty, new types_1.Balance(channelBalance.balance.sub(channelBalance.balance_a)));
                    }
                }
                else {
                    if (channelBalance.balance_a.gtn(0)) {
                        await this.channel.increaseFunds(counterparty, channelBalance.balance_a);
                    }
                }
                // listen for opening event and update DB
                onceOpen(this, this.account, counterparty).then(() => onOpen(this, counterpartyPubKey, signedChannel));
                yield signedChannel.toU8a();
            }
        }.call(this);
    }
}
exports.default = Channel;
