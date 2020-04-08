"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const assert_1 = __importDefault(require("assert"));
const peer_id_1 = __importDefault(require("peer-id"));
const _1 = require(".");
const challenge_1 = require("../packet/challenge");
const utils_1 = require("../../utils");
const bn_js_1 = __importDefault(require("bn.js"));
const hopr_core_polkadot_1 = require("@hoprnet/hopr-core-polkadot");
const crypto_1 = require("crypto");
const secp256k1_1 = __importDefault(require("secp256k1"));
describe('test acknowledgement generation', function () {
    it('should generate a valid acknowledgement', async function () {
        const paymentChannels = {
            utils: hopr_core_polkadot_1.Utils,
            types: hopr_core_polkadot_1.Types
        };
        const sender = await peer_id_1.default.create({
            keyType: 'secp256k1'
        });
        const receiver = await peer_id_1.default.create({
            keyType: 'secp256k1'
        });
        const secret = crypto_1.randomBytes(32);
        const challenge = await challenge_1.Challenge.create(paymentChannels, secret, new bn_js_1.default(0)).sign(sender);
        assert_1.default(await challenge.verify(sender), `Previously generated challenge should be valid.`);
        const pubKey = sender.pubKey.marshal();
        assert_1.default(utils_1.u8aEquals(await challenge.counterparty, pubKey), `recovered pubKey should be equal.`);
        const ack = await _1.Acknowledgement.create(paymentChannels, challenge, secp256k1_1.default.publicKeyCreate(secret), receiver);
        assert_1.default(await ack.verify(receiver), `Previously generated acknowledgement should be valid.`);
        assert_1.default(utils_1.u8aEquals(await ack.responseSigningParty, receiver.pubKey.marshal()), `recovered pubKey should be equal.`);
    });
});
