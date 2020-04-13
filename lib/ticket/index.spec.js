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
const hopr_utils_1 = require("@hoprnet/hopr-utils");
const HoprToken_json_1 = __importDefault(require("@hoprnet/hopr-ethereum/build/extracted/abis/HoprToken.json"));
const testing_1 = require("../utils/testing");
const crypto_1 = require("crypto");
const bn_js_1 = __importDefault(require("bn.js"));
const it_pipe_1 = __importDefault(require("it-pipe"));
const web3_1 = __importDefault(require("web3"));
const types_1 = require("../types");
const channel_1 = require("../types/channel");
const channel_2 = __importDefault(require("../channel"));
const _1 = __importDefault(require("."));
const configs = __importStar(require("../config"));
describe('test ticket generation and verification', function () {
    const web3 = new web3_1.default(configs.DEFAULT_URI);
    const hoprToken = new web3.eth.Contract(HoprToken_json_1.default, configs.TOKEN_ADDRESSES.private);
    let coreConnector;
    let counterpartysCoreConnector;
    let funder;
    beforeEach(async function () {
        funder = await testing_1.getPrivKeyData(hopr_utils_1.stringToU8a(configs.FUND_ACCOUNT_PRIVATE_KEY));
        const userA = await testing_1.generateUser(web3, funder, hoprToken);
        const userB = await testing_1.generateUser(web3, funder, hoprToken);
        coreConnector = await testing_1.generateNode(userA.privKey);
        counterpartysCoreConnector = await testing_1.generateNode(userB.privKey);
        await coreConnector.db.clear();
        await counterpartysCoreConnector.db.clear();
    });
    it('should store ticket', async function () {
        const channelType = new types_1.Channel(undefined, {
            balance: new types_1.ChannelBalance(undefined, {
                balance: new bn_js_1.default(123),
                balance_a: new bn_js_1.default(122)
            }),
            status: channel_1.ChannelStatus.FUNDING
        });
        const channelId = new types_1.Hash(await coreConnector.utils.getId(coreConnector.self.onChainKeyPair.publicKey, counterpartysCoreConnector.self.onChainKeyPair.publicKey));
        const signedChannel = await types_1.SignedChannel.create(counterpartysCoreConnector, undefined, { channel: channelType });
        const channel = await channel_2.default.create(coreConnector, counterpartysCoreConnector.self.publicKey, async () => counterpartysCoreConnector.self.onChainKeyPair.publicKey, signedChannel.channel.balance, async () => {
            const result = await it_pipe_1.default([(await types_1.SignedChannel.create(coreConnector, undefined, { channel: channelType })).subarray()], channel_2.default.handleOpeningRequest(counterpartysCoreConnector), async (source) => {
                let result;
                for await (const msg of source) {
                    if (result == null) {
                        result = msg.slice();
                        return result;
                    }
                    else {
                        continue;
                    }
                }
            });
            return new types_1.SignedChannel({
                bytes: result.buffer,
                offset: result.byteOffset
            });
        });
        const preImage = crypto_1.randomBytes(32);
        const hash = await coreConnector.utils.hash(preImage);
        const signedTicket = await channel.ticket.create(channel, new types_1.Balance(1), new types_1.Hash(hash));
        assert_1.default(hopr_utils_1.u8aEquals(await signedTicket.signer, coreConnector.self.publicKey), `Check that signer is recoverable`);
        const signedChannelCounterparty = await types_1.SignedChannel.create(coreConnector, undefined, { channel: channelType });
        assert_1.default(hopr_utils_1.u8aEquals(await signedChannelCounterparty.signer, coreConnector.self.publicKey), `Check that signer is recoverable.`);
        await _1.default.store(coreConnector, channelId, signedTicket);
        const storedSignedTicket = new Uint8Array(await coreConnector.db.get(Buffer.from(coreConnector.dbKeys.Ticket(channelId, signedTicket.ticket.challenge))));
        assert_1.default(hopr_utils_1.u8aEquals(signedTicket, storedSignedTicket), `Check that signedTicket is stored correctly`);
    });
    it('should store tickets, and retrieve them in a map', async function () {
        const channelType = new types_1.Channel(undefined, {
            balance: new types_1.ChannelBalance(undefined, {
                balance: new bn_js_1.default(123),
                balance_a: new bn_js_1.default(122)
            }),
            status: channel_1.ChannelStatus.FUNDING
        });
        const channelId = new types_1.Hash(await coreConnector.utils.getId(coreConnector.self.onChainKeyPair.publicKey, counterpartysCoreConnector.self.onChainKeyPair.publicKey));
        const signedChannel = await types_1.SignedChannel.create(counterpartysCoreConnector, undefined, { channel: channelType });
        const channel = await channel_2.default.create(coreConnector, counterpartysCoreConnector.self.publicKey, async () => counterpartysCoreConnector.self.onChainKeyPair.publicKey, signedChannel.channel.balance, async () => {
            const result = await it_pipe_1.default([(await types_1.SignedChannel.create(coreConnector, undefined, { channel: channelType })).subarray()], channel_2.default.handleOpeningRequest(counterpartysCoreConnector), async (source) => {
                let result;
                for await (const msg of source) {
                    if (result == null) {
                        result = msg.slice();
                        return result;
                    }
                    else {
                        continue;
                    }
                }
            });
            return new types_1.SignedChannel({
                bytes: result.buffer,
                offset: result.byteOffset
            });
        });
        const hashA = await coreConnector.utils.hash(crypto_1.randomBytes(32));
        const hashB = await coreConnector.utils.hash(crypto_1.randomBytes(32));
        const signedTicketA = await channel.ticket.create(channel, new types_1.Balance(1), new types_1.Hash(hashA));
        const signedTicketB = await channel.ticket.create(channel, new types_1.Balance(1), new types_1.Hash(hashB));
        await Promise.all([
            _1.default.store(coreConnector, channelId, signedTicketA),
            _1.default.store(coreConnector, channelId, signedTicketB),
            _1.default.store(coreConnector, new Uint8Array(types_1.Hash.SIZE).fill(0x00), signedTicketB)
        ]);
        const storedSignedTickets = await _1.default.get(coreConnector, channelId);
        assert_1.default(storedSignedTickets.size === 2, `Check getting signedTickets`);
        const storedSignedTicketA = storedSignedTickets.get(hopr_utils_1.u8aToHex(signedTicketA.ticket.challenge));
        assert_1.default(hopr_utils_1.u8aEquals(signedTicketA, storedSignedTicketA), `Check that signedTicketA is stored correctly`);
        const storedSignedTicketB = storedSignedTickets.get(hopr_utils_1.u8aToHex(signedTicketB.ticket.challenge));
        assert_1.default(hopr_utils_1.u8aEquals(signedTicketB, storedSignedTicketB), `Check that signedTicketB is stored correctly`);
    });
});
