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
describe('test signedTicket construction', function () {
    it('should create new signedTicket using struct', async function () {
        const ticketData = await generateTicketData();
        const ticket = new _1.Ticket(undefined, ticketData);
        const signature = await utils.sign(await ticket.hash, userA).then(res => {
            return new _1.Signature({
                bytes: res.buffer,
                offset: res.byteOffset
            });
        });
        const signedTicket = new _1.SignedTicket(undefined, {
            signature,
            ticket
        });
        assert_1.default(signedTicket.ticket.channelId.eq(ticketData.channelId), 'wrong channelId');
        assert_1.default(signedTicket.ticket.challenge.eq(ticketData.challenge), 'wrong challenge');
        assert_1.default(signedTicket.ticket.epoch.eq(ticketData.epoch), 'wrong epoch');
        assert_1.default(signedTicket.ticket.amount.eq(ticketData.amount), 'wrong amount');
        assert_1.default(signedTicket.ticket.winProb.eq(ticketData.winProb), 'wrong winProb');
        assert_1.default(signedTicket.ticket.onChainSecret.eq(ticketData.onChainSecret), 'wrong onChainSecret');
    });
    it('should create new signedTicket using array', async function () {
        const ticketData = await generateTicketData();
        const ticket = new _1.Ticket(undefined, ticketData);
        const signature = await utils.sign(await ticket.hash, userA).then(res => {
            return new _1.Signature({
                bytes: res.buffer,
                offset: res.byteOffset
            });
        });
        const signedTicketA = new _1.SignedTicket(undefined, {
            signature,
            ticket
        });
        const signedTicketB = new _1.SignedTicket({
            bytes: signedTicketA.buffer,
            offset: signedTicketA.byteOffset
        });
        assert_1.default(signedTicketB.ticket.channelId.eq(ticketData.channelId), 'wrong channelId');
        assert_1.default(signedTicketB.ticket.challenge.eq(ticketData.challenge), 'wrong challenge');
        assert_1.default(signedTicketB.ticket.epoch.eq(ticketData.epoch), 'wrong epoch');
        assert_1.default(signedTicketB.ticket.amount.eq(ticketData.amount), 'wrong amount');
        assert_1.default(signedTicketB.ticket.winProb.eq(ticketData.winProb), 'wrong winProb');
        assert_1.default(signedTicketB.ticket.onChainSecret.eq(ticketData.onChainSecret), 'wrong onChainSecret');
    });
    it('should verify signedTicket', async function () {
        const ticketData = await generateTicketData();
        const ticket = new _1.Ticket(undefined, ticketData);
        const signature = await utils.sign(await ticket.hash, userA).then(res => {
            return new _1.Signature({
                bytes: res.buffer,
                offset: res.byteOffset
            });
        });
        const signedTicket = new _1.SignedTicket(undefined, {
            signature,
            ticket
        });
        const signer = new _1.Hash(await signedTicket.signer);
        const userAPubKey = await utils.privKeyToPubKey(userA);
        assert_1.default(signer.eq(userAPubKey), 'signer incorrect');
    });
});
