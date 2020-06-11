"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.ChannelFactory = void 0;
const hopr_utils_1 = require("@hoprnet/hopr-utils");
const bn_js_1 = __importDefault(require("bn.js"));
const types_1 = require("../types");
const channel_1 = require("../types/channel");
const constants_1 = require("../constants");
const utils_1 = require("../utils");
const channel_2 = __importDefault(require("./channel"));
const extended_1 = require("../types/extended");
const crypto_1 = require("crypto");
const constants_2 = require("./constants");
const WIN_PROB = new bn_js_1.default(1);
class ChannelFactory {
    constructor(coreConnector) {
        this.coreConnector = coreConnector;
    }
    async increaseFunds(counterparty, amount) {
        try {
            if ((await this.coreConnector.account.balance).lt(amount)) {
                throw Error(constants_1.ERRORS.OOF_HOPR);
            }
            await utils_1.waitForConfirmation((await this.coreConnector.signTransaction(this.coreConnector.hoprToken.methods.send(this.coreConnector.hoprChannels.options.address, amount.toString(), this.coreConnector.web3.eth.abi.encodeParameters(['address', 'address'], [(await this.coreConnector.account.address).toHex(), counterparty.toHex()])), {
                from: (await this.coreConnector.account.address).toHex(),
                to: this.coreConnector.hoprToken.options.address,
                nonce: await this.coreConnector.account.nonce,
            })).send());
        }
        catch (error) {
            throw error;
        }
    }
    async isOpen(counterpartyPubKey) {
        const counterparty = await this.coreConnector.utils.pubKeyToAccountId(counterpartyPubKey);
        const channelId = await this.coreConnector.utils
            .getId(await this.coreConnector.account.address, counterparty)
            .then((res) => new types_1.Hash(res));
        const [onChain, offChain] = await Promise.all([
            this.coreConnector.channel.getOnChainState(channelId).then((channel) => {
                const state = Number(channel.stateCounter) % constants_2.CHANNEL_STATES;
                return state === channel_1.ChannelStatus.OPEN || state === channel_1.ChannelStatus.PENDING;
            }),
            this.coreConnector.db.get(Buffer.from(this.coreConnector.dbKeys.Channel(counterpartyPubKey))).then(() => true, (err) => {
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
                this.coreConnector.log(`Channel ${hopr_utils_1.u8aToHex(channelId)} exists off-chain but not on-chain, deleting data.`);
                await this.coreConnector.channel.deleteOffChainState(counterpartyPubKey);
            }
            else {
                throw Error(`Channel ${hopr_utils_1.u8aToHex(channelId)} exists on-chain but not off-chain.`);
            }
        }
        return onChain && offChain;
    }
    async createDummyChannelTicket(counterParty, challenge, arr) {
        if (!challenge) {
            throw Error(`Challenge is not set`);
        }
        const channelId = await this.coreConnector.utils.getId(await this.coreConnector.utils.pubKeyToAccountId(this.coreConnector.account.keys.onChain.pubKey), counterParty);
        const winProb = new extended_1.Uint8ArrayE(new bn_js_1.default(new Uint8Array(types_1.Hash.SIZE).fill(0xff)).div(WIN_PROB).toArray('le', types_1.Hash.SIZE));
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
        await this.coreConnector.utils.sign(await ticket.hash, this.coreConnector.account.keys.onChain.privKey, undefined, {
            bytes: signedTicket.buffer,
            offset: signedTicket.signatureOffset,
        });
        return signedTicket;
    }
    async createSignedChannel(arr, struct) {
        const emptySignatureArray = new Uint8Array(types_1.Signature.SIZE).fill(0x00);
        let signedChannel;
        if (typeof arr !== 'undefined') {
            signedChannel = new types_1.SignedChannel(arr);
        }
        else if (typeof struct !== 'undefined') {
            signedChannel = new types_1.SignedChannel(undefined, {
                channel: struct.channel,
                signature: struct.signature ||
                    new types_1.Signature({
                        bytes: emptySignatureArray.buffer,
                        offset: emptySignatureArray.byteOffset,
                    }),
            });
        }
        else {
            throw Error(`Invalid input parameters.`);
        }
        if (signedChannel.signature.eq(emptySignatureArray)) {
            await struct.channel.sign(this.coreConnector.account.keys.onChain.privKey, undefined, {
                bytes: signedChannel.buffer,
                offset: signedChannel.signatureOffset,
            });
        }
        return signedChannel;
    }
    async create(counterpartyPubKey, _getOnChainPublicKey, channelBalance, sign) {
        const counterparty = await this.coreConnector.utils.pubKeyToAccountId(counterpartyPubKey);
        let channel;
        let signedChannel;
        if (!this.coreConnector._onChainValuesInitialized) {
            await this.coreConnector.initOnchainValues();
        }
        if (await this.isOpen(counterpartyPubKey)) {
            const record = await this.coreConnector.db.get(Buffer.from(this.coreConnector.dbKeys.Channel(counterpartyPubKey)));
            signedChannel = new types_1.SignedChannel({
                bytes: record.buffer,
                offset: record.byteOffset,
            });
            channel = new channel_2.default(this.coreConnector, counterpartyPubKey, signedChannel);
        }
        else if (sign != null && channelBalance != null) {
            let amount;
            if (this.coreConnector.utils.isPartyA(await this.coreConnector.account.address, counterparty)) {
                amount = channelBalance.balance_a;
            }
            else {
                amount = new types_1.Balance(channelBalance.balance.sub(channelBalance.balance_a));
            }
            await this.increaseFunds(counterparty, amount);
            signedChannel = await sign(channelBalance);
            channel = new channel_2.default(this.coreConnector, counterpartyPubKey, signedChannel);
            await utils_1.waitForConfirmation((await this.coreConnector.signTransaction(this.coreConnector.hoprChannels.methods.openChannel(counterparty.toHex()), {
                from: (await this.coreConnector.account.address).toHex(),
                to: this.coreConnector.hoprChannels.options.address,
                nonce: await this.coreConnector.account.nonce,
            })).send());
            await this.coreConnector.db.put(Buffer.from(this.coreConnector.dbKeys.Channel(counterpartyPubKey)), Buffer.from(signedChannel));
        }
        else {
            throw Error('Invalid input parameters.');
        }
        return channel;
    }
    getAll(onData, onEnd) {
        const promises = [];
        return new Promise((resolve, reject) => {
            this.coreConnector.db
                .createReadStream({
                gte: Buffer.from(this.coreConnector.dbKeys.Channel(new Uint8Array(types_1.Hash.SIZE).fill(0x00))),
                lte: Buffer.from(this.coreConnector.dbKeys.Channel(new Uint8Array(types_1.Hash.SIZE).fill(0xff))),
            })
                .on('error', (err) => reject(err))
                .on('data', ({ key, value }) => {
                const signedChannel = new types_1.SignedChannel({
                    bytes: value.buffer,
                    offset: value.byteOffset,
                });
                promises.push(onData(new channel_2.default(this.coreConnector, this.coreConnector.dbKeys.ChannelKeyParse(key), signedChannel)));
            })
                .on('end', () => resolve(onEnd(promises)));
        });
    }
    async closeChannels() {
        const result = new bn_js_1.default(0);
        return this.getAll((channel) => channel.initiateSettlement().then(() => {
            // @TODO: add balance
            result.iaddn(0);
        }), async (promises) => {
            await Promise.all(promises);
            return new types_1.Balance(result);
        });
    }
    handleOpeningRequest() {
        return (source) => async function* () {
            for await (const _msg of source) {
                const msg = _msg.slice();
                const signedChannel = new types_1.SignedChannel({
                    bytes: msg.buffer,
                    offset: msg.byteOffset,
                });
                const counterpartyPubKey = await signedChannel.signer;
                const counterparty = await this.coreConnector.utils.pubKeyToAccountId(counterpartyPubKey);
                const channelBalance = signedChannel.channel.balance;
                if (this.coreConnector.utils.isPartyA(await this.coreConnector.account.address, counterparty)) {
                    if (channelBalance.balance.sub(channelBalance.balance_a).gtn(0)) {
                        await this.increaseFunds(counterparty, new types_1.Balance(channelBalance.balance.sub(channelBalance.balance_a)));
                    }
                }
                else {
                    if (channelBalance.balance_a.gtn(0)) {
                        await this.increaseFunds(counterparty, channelBalance.balance_a);
                    }
                }
                // listen for opening event and update DB
                this.coreConnector.channel
                    .onceOpen(await this.coreConnector.account.address, counterparty)
                    .then(() => this.coreConnector.channel.saveOffChainState(counterpartyPubKey, signedChannel));
                yield signedChannel.toU8a();
            }
        }.call(this);
    }
    getOffChainState(counterparty) {
        return this.coreConnector.db.get(Buffer.from(this.coreConnector.dbKeys.Channel(counterparty)));
    }
    saveOffChainState(counterparty, signedChannel) {
        return this.coreConnector.db.put(Buffer.from(this.coreConnector.dbKeys.Channel(counterparty)), Buffer.from(signedChannel));
    }
    deleteOffChainState(counterparty) {
        return this.coreConnector.db.del(Buffer.from(this.coreConnector.dbKeys.Channel(counterparty)));
    }
    getOnChainState(channelId) {
        return this.coreConnector.hoprChannels.methods.channels(channelId.toHex()).call();
    }
    async onceOpen(self, counterparty) {
        const channelId = await utils_1.getId(self, counterparty);
        return utils_1.cleanupPromiEvent(this.coreConnector.hoprChannels.events.OpenedChannel({
            filter: {
                opener: [self.toHex(), counterparty.toHex()],
                counterParty: [self.toHex(), counterparty.toHex()],
            },
        }), (event) => {
            return new Promise((resolve, reject) => {
                event
                    .on('data', async (data) => {
                    const { opener, counterParty } = data.returnValues;
                    const _channelId = await this.coreConnector.utils.getId(new types_1.AccountId(hopr_utils_1.stringToU8a(opener)), new types_1.AccountId(hopr_utils_1.stringToU8a(counterParty)));
                    if (!hopr_utils_1.u8aEquals(_channelId, channelId)) {
                        return;
                    }
                    return resolve(data.returnValues);
                })
                    .on('error', reject);
            });
        });
    }
    async onceClosed(self, counterparty) {
        const channelId = await utils_1.getId(self, counterparty);
        return utils_1.cleanupPromiEvent(this.coreConnector.hoprChannels.events.ClosedChannel({
            filter: {
                closer: [self.toHex(), counterparty.toHex()],
                counterParty: [self.toHex(), counterparty.toHex()],
            },
        }), (event) => {
            return new Promise((resolve, reject) => {
                event
                    .on('data', async (data) => {
                    const { closer, counterParty } = data.returnValues;
                    const _channelId = await this.coreConnector.utils.getId(new types_1.AccountId(hopr_utils_1.stringToU8a(closer)), new types_1.AccountId(hopr_utils_1.stringToU8a(counterParty)));
                    if (!hopr_utils_1.u8aEquals(_channelId, channelId)) {
                        return;
                    }
                    resolve(data.returnValues);
                })
                    .on('error', reject);
            });
        });
    }
}
exports.ChannelFactory = ChannelFactory;
exports.default = channel_2.default;
//# sourceMappingURL=index.js.map