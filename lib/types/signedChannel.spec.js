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
const bn_js_1 = __importDefault(require("bn.js"));
const _1 = require(".");
const channel_1 = require("./channel");
const u8a = __importStar(require("../core/u8a"));
const utils = __importStar(require("../utils"));
const config_1 = require("../config");
const [userA] = config_1.DEMO_ACCOUNTS.map(str => u8a.stringToU8a(str));
const generateChannelData = async () => {
    const balance = new _1.ChannelBalance(undefined, {
        balance: new bn_js_1.default(10),
        balance_a: new bn_js_1.default(2)
    });
    const status = channel_1.ChannelStatus.UNINITIALISED;
    return {
        balance,
        status
    };
};
describe('test signedChannel construction', function () {
    it('should create new signedChannel using struct', async function () {
        const channelData = await generateChannelData();
        const channel = new _1.Channel(undefined, channelData);
        const signature = await utils.sign(await channel.hash, userA).then(res => {
            return new _1.Signature({
                bytes: res.buffer,
                offset: res.byteOffset
            });
        });
        const signedChannel = new _1.SignedChannel(undefined, {
            signature,
            channel
        });
        assert_1.default(signedChannel.channel.balance.eq(channelData.balance), 'wrong balance');
        assert_1.default(new bn_js_1.default(signedChannel.channel.status).eq(new bn_js_1.default(channelData.status)), 'wrong status');
    });
    it('should create new signedChannel using array', async function () {
        const channelData = await generateChannelData();
        const channel = new _1.Channel(undefined, channelData);
        const signature = await utils.sign(await channel.hash, userA).then(res => {
            return new _1.Signature({
                bytes: res.buffer,
                offset: res.byteOffset
            });
        });
        const signedChannelA = new _1.SignedChannel(undefined, {
            signature,
            channel
        });
        const signedChannelB = new _1.SignedChannel({
            bytes: signedChannelA.buffer,
            offset: signedChannelA.byteOffset
        });
        assert_1.default(signedChannelB.channel.balance.eq(channelData.balance), 'wrong balance');
        assert_1.default(new bn_js_1.default(signedChannelB.channel.status).eq(new bn_js_1.default(channelData.status)), 'wrong status');
    });
    it('should verify signedChannel', async function () {
        const channelData = await generateChannelData();
        const channel = new _1.Channel(undefined, channelData);
        const signature = await utils.sign(await channel.hash, userA).then(res => {
            return new _1.Signature({
                bytes: res.buffer,
                offset: res.byteOffset
            });
        });
        const signedChannel = new _1.SignedChannel(undefined, {
            signature,
            channel
        });
        const signer = new _1.Hash(await signedChannel.signer);
        const userAPubKey = await utils.privKeyToPubKey(userA);
        assert_1.default(signer.eq(userAPubKey), 'signer incorrect');
    });
});
