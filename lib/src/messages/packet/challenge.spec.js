"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const assert_1 = __importDefault(require("assert"));
const challenge_1 = require("./challenge");
const hopr_core_polkadot_1 = require("@hoprnet/hopr-core-polkadot");
const bn_js_1 = __importDefault(require("bn.js"));
const peer_id_1 = __importDefault(require("peer-id"));
const crypto_1 = require("crypto");
const utils_1 = require("../../utils");
describe('test creation & verification of a challenge', function () {
    it('should create a verifiable challenge', async function () {
        const paymentChannels = {
            utils: hopr_core_polkadot_1.Utils,
            types: hopr_core_polkadot_1.Types
        };
        const secret = crypto_1.randomBytes(32);
        const peerId = await peer_id_1.default.create({
            keyType: 'secp256k1'
        });
        const challenge = await challenge_1.Challenge.create(paymentChannels, secret, new bn_js_1.default(0)).sign(peerId);
        assert_1.default(await challenge.verify(peerId), `Previously generated signature should be valid.`);
        assert_1.default(utils_1.u8aEquals(await challenge.counterparty, peerId.pubKey.marshal()), `recovered pubKey should be equal.`);
        challenge[0] ^= 0xff;
        try {
            await challenge.verify(peerId);
            assert_1.default.fail(`Manipulated signature should be with high probability invalid.`);
        }
        catch { }
    });
});
