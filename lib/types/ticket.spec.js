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
const crypto_1 = require("crypto");
const bn_js_1 = __importDefault(require("bn.js"));
const _1 = require(".");
const u8a = __importStar(require("../core/u8a"));
const utils = __importStar(require("../utils"));
const config_1 = require("../config");
const [userA, userB] = config_1.DEMO_ACCOUNTS.map(str => u8a.stringToU8a(str));
const WIN_PROB = new bn_js_1.default(1);
const generateTicketData = async () => {
    const channelId = new _1.Hash(await utils.getId(userA, userB));
    const challenge = new _1.Hash(crypto_1.randomBytes(32));
    const epoch = new _1.TicketEpoch(0);
    const amount = new _1.Balance(15);
    const winProb = new _1.Hash(new bn_js_1.default(new Uint8Array(_1.Hash.SIZE).fill(0xff)).div(WIN_PROB).toArray('le', _1.Hash.SIZE));
    const onChainSecret = new _1.Hash(crypto_1.randomBytes(32));
    return {
        channelId,
        challenge,
        epoch,
        amount,
        winProb,
        onChainSecret
    };
};
describe('test ticket construction', function () {
    it('should create new ticket using struct', async function () {
        const ticketData = await generateTicketData();
        const ticket = new _1.Ticket(undefined, ticketData);
        assert_1.default(ticket.channelId.eq(ticketData.channelId), 'wrong channelId');
        assert_1.default(ticket.challenge.eq(ticketData.challenge), 'wrong challenge');
        assert_1.default(ticket.epoch.eq(ticketData.epoch), 'wrong epoch');
        assert_1.default(ticket.amount.eq(ticketData.amount), 'wrong amount');
        assert_1.default(ticket.winProb.eq(ticketData.winProb), 'wrong winProb');
        assert_1.default(ticket.onChainSecret.eq(ticketData.onChainSecret), 'wrong onChainSecret');
    });
    it('should create new ticket using array', async function () {
        const ticketData = await generateTicketData();
        const ticketA = new _1.Ticket(undefined, ticketData);
        const ticketB = new _1.Ticket({
            bytes: ticketA.buffer,
            offset: ticketA.byteOffset
        });
        assert_1.default(ticketB.channelId.eq(ticketData.channelId), 'wrong channelId');
        assert_1.default(ticketB.challenge.eq(ticketData.challenge), 'wrong challenge');
        assert_1.default(ticketB.epoch.eq(ticketData.epoch), 'wrong epoch');
        assert_1.default(ticketB.amount.eq(ticketData.amount), 'wrong amount');
        assert_1.default(ticketB.winProb.eq(ticketData.winProb), 'wrong winProb');
        assert_1.default(ticketB.onChainSecret.eq(ticketData.onChainSecret), 'wrong onChainSecret');
    });
});
