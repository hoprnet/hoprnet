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
const _1 = __importDefault(require("."));
const configs = __importStar(require("../config"));
describe('test ticket generation and verification', function () {
    const web3 = new web3_1.default(configs.DEFAULT_URI);
    const hoprToken = new web3.eth.Contract(HoprToken_json_1.default, configs.TOKEN_ADDRESSES.private);
    const channels = new Map();
    const preChannels = new Map();
    let coreConnector;
    let counterpartysCoreConnector;
    let funder;
    beforeEach(async function () {
        channels.clear();
        preChannels.clear();
        funder = await testing_1.getPrivKeyData(hopr_utils_1.stringToU8a(configs.FUND_ACCOUNT_PRIVATE_KEY));
        const userA = await testing_1.generateUser(web3, funder, hoprToken);
        const userB = await testing_1.generateUser(web3, funder, hoprToken);
        coreConnector = await testing_1.generateNode(userA.privKey);
        counterpartysCoreConnector = await testing_1.generateNode(userB.privKey);
    });
    it('should create a valid ticket', async function () {
        const channelType = new types_1.Channel(undefined, {
            balance: new types_1.ChannelBalance(undefined, {
                balance: new bn_js_1.default(123),
                balance_a: new bn_js_1.default(122)
            }),
            status: channel_1.ChannelStatus.FUNDING
        });
        const channelId = await coreConnector.utils.getId(coreConnector.self.onChainKeyPair.publicKey, counterpartysCoreConnector.self.onChainKeyPair.publicKey);
        const signedChannel = await types_1.SignedChannel.create(counterpartysCoreConnector, undefined, { channel: channelType });
        preChannels.set(hopr_utils_1.u8aToHex(channelId), channelType);
        const channel = await _1.default.create(coreConnector, counterpartysCoreConnector.self.publicKey, async () => counterpartysCoreConnector.self.onChainKeyPair.publicKey, signedChannel.channel.balance, async () => {
            const result = await it_pipe_1.default([(await types_1.SignedChannel.create(coreConnector, undefined, { channel: channelType })).subarray()], _1.default.handleOpeningRequest(counterpartysCoreConnector), async (source) => {
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
        channels.set(hopr_utils_1.u8aToHex(channelId), channelType);
        const preImage = crypto_1.randomBytes(32);
        const hash = await coreConnector.utils.hash(preImage);
        const ticket = await channel.ticket.create(channel, new types_1.Balance(1), new types_1.Hash(hash));
        assert_1.default(hopr_utils_1.u8aEquals(await ticket.signer, coreConnector.self.publicKey), `Check that signer is recoverable`);
        const signedChannelCounterparty = await types_1.SignedChannel.create(coreConnector, undefined, { channel: channelType });
        assert_1.default(hopr_utils_1.u8aEquals(await signedChannelCounterparty.signer, coreConnector.self.publicKey), `Check that signer is recoverable.`);
        counterpartysCoreConnector.db.put(Buffer.from(coreConnector.dbKeys.Channel(coreConnector.self.onChainKeyPair.publicKey)), Buffer.from(signedChannelCounterparty));
        const dbChannels = (await counterpartysCoreConnector.channel.getAll(counterpartysCoreConnector, async (arg) => arg, async (arg) => Promise.all(arg)));
        assert_1.default(hopr_utils_1.u8aEquals(dbChannels[0].counterparty, coreConnector.self.onChainKeyPair.publicKey), `Channel record should make it into the database and its db-key should lead to the AccountId of the counterparty.`);
        const counterpartysChannel = await _1.default.create(counterpartysCoreConnector, coreConnector.self.publicKey, () => Promise.resolve(coreConnector.self.onChainKeyPair.publicKey), signedChannel.channel.balance, () => Promise.resolve(signedChannelCounterparty));
        assert_1.default(await coreConnector.channel.isOpen(coreConnector, counterpartysCoreConnector.self.onChainKeyPair.publicKey), `Checks that party A considers the channel open.`);
        assert_1.default(await counterpartysCoreConnector.channel.isOpen(counterpartysCoreConnector, coreConnector.self.onChainKeyPair.publicKey), `Checks that party B considers the channel open.`);
        await channel.testAndSetNonce(new Uint8Array(1).fill(0xff)), `Should be able to set nonce.`;
        assert_1.default.rejects(() => channel.testAndSetNonce(new Uint8Array(1).fill(0xff)), `Should reject when trying to set nonce twice.`);
        assert_1.default(await counterpartysChannel.ticket.verify(counterpartysChannel, ticket), `Ticket signature must be valid.`);
    });
});
